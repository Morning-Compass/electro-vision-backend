use std::any::Any;
use std::env;

use chrono::{Duration, NaiveDateTime, TimeDelta, Utc};
use diesel::dsl::exists;
use diesel::prelude::OptionalExtension;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use dotenvy::dotenv;
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::Transport;

use crate::constants::SMTP;
use crate::schema::confirmation_tokens::dsl as ct_table;
use crate::schema::password_reset_tokens::dsl as psr_table;
use crate::schema::users as user_data;
use crate::schema::users::dsl as user_table;
use schema::confirmation_tokens as ct_data;
use schema::password_reset_tokens as psr_data;

use crate::auth::auth_error::{
    AccountVerification, VerificationTokenError, VerificationTokenServerError,
};
use crate::emails::{email_body_generator, EmailType};
use crate::schema::users::account_valid;
use crate::schema::{confirmation_tokens, password_reset_tokens};
use crate::{constants::CONFIRMATION_TOKEN_EXIPIRATION_TIME, est_conn, DPool};
use crate::{models, schema, user};

pub enum TokenEmailType {
    AccountVerification,
    AccountVerificationResend,
    PasswordReset,
    PasswordResetResend,
}

pub enum TokenType {
    AccountVerification,
    PasswordReset(String), //str email
}

pub struct Cft {
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

pub trait ConfirmationToken {
    fn new(
        email: String,
        resend: bool,
        token_type: TokenType,
        pool: DPool,
    ) -> Result<String, VerificationTokenError>;
    fn confirm(
        token: String,
        token_type: TokenType,
        pool: DPool,
    ) -> Result<String, VerificationTokenError>;
    async fn send(
        username: String,
        email: String,
        pool: DPool,
        email_type: TokenEmailType,
        token: Option<String>,
        resend: bool,
        token_type: TokenType,
    ) -> Result<String, VerificationTokenError>;
}

impl ConfirmationToken for Cft {
    fn new(
        u_email: String,
        resend: bool,
        token_type: TokenType,
        pool: DPool,
    ) -> Result<String, VerificationTokenError> {
        let mut token_exists: bool = false;

        match token_type {
            TokenType::AccountVerification => {
                if !resend {
                    token_exists = diesel::select(exists(
                        ct_table::confirmation_tokens.filter(ct_data::user_email.eq(&u_email)),
                    ))
                    .get_result::<bool>(&mut est_conn(pool.clone()))
                    .map_err(|_| {
                        VerificationTokenError::ServerError(
                            VerificationTokenServerError::DatabaseError,
                        )
                    })?;
                }
            }
            TokenType::PasswordReset(_) => {
                if !resend {
                    token_exists = diesel::select(exists(
                        psr_table::password_reset_tokens.filter(psr_data::user_email.eq(&u_email)),
                    ))
                    .get_result::<bool>(&mut est_conn(pool.clone()))
                    .map_err(|_| {
                        VerificationTokenError::ServerError(
                            VerificationTokenServerError::DatabaseError,
                        )
                    })?;
                }
            }
        }

        if token_exists {
            return Err(VerificationTokenError::TokenAlreadyExists);
        }

        let ctoken = Cft {
            user_email: u_email.clone(),
            token: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: Utc::now()
                .naive_utc()
                .checked_add_signed(TimeDelta::seconds(CONFIRMATION_TOKEN_EXIPIRATION_TIME))
                .ok_or_else(|| {
                    VerificationTokenError::ServerError(
                        VerificationTokenServerError::SettingExpirationDateError,
                    )
                })?,
        };

        let token_to_return = ctoken.token.clone();

        match token_type {
            TokenType::AccountVerification => {
                match diesel::insert_into(ct_table::confirmation_tokens)
                    .values((
                        ct_data::user_email.eq(ctoken.user_email),
                        ct_data::token.eq(ctoken.token),
                        ct_data::created_at.eq(ctoken.created_at),
                        ct_data::expires_at.eq(ctoken.expires_at),
                    ))
                    .execute(&mut est_conn(pool))
                {
                    Ok(_) => Ok(token_to_return),
                    Err(_) => Err(VerificationTokenError::ServerError(
                        VerificationTokenServerError::TokenInsertionError,
                    )),
                }
            }
            TokenType::PasswordReset(_) => {
                match diesel::insert_into(psr_table::password_reset_tokens)
                    .values((
                        psr_data::user_email.eq(ctoken.user_email),
                        psr_data::token.eq(ctoken.token),
                        psr_data::created_at.eq(ctoken.created_at),
                        psr_data::expires_at.eq(ctoken.expires_at),
                    ))
                    .execute(&mut est_conn(pool))
                {
                    Ok(_) => Ok(token_to_return),
                    Err(_) => Err(VerificationTokenError::ServerError(
                        VerificationTokenServerError::TokenInsertionError,
                    )),
                }
            }
        }
    }

    fn confirm(
        _token: String,
        token_type: TokenType,
        pool: DPool,
    ) -> Result<String, VerificationTokenError> {
        use crate::schema::confirmation_tokens::dsl::*;
        let mut conn = est_conn(pool);

        enum UnifiedToken {
            Confirmation(models::ConfirmationToken),
            PasswordReset(models::PasswordResetTokens, String),
        }

        type Token = Result<Option<UnifiedToken>, diesel::result::Error>;

        let db_token: Token = match token_type {
            TokenType::AccountVerification => ct_table::confirmation_tokens
                .filter(ct_data::token.eq(&_token))
                .first::<models::ConfirmationToken>(&mut conn)
                .optional()
                .map(|opt| opt.map(|ct| UnifiedToken::Confirmation(ct))),
            TokenType::PasswordReset(mail) => psr_table::password_reset_tokens
                .filter(psr_data::token.eq(&_token))
                .first::<models::PasswordResetTokens>(&mut conn)
                .optional()
                .map(|opt| opt.map(|ct| UnifiedToken::PasswordReset(ct, mail))),
        };

        match db_token {
            Ok(Some(UnifiedToken::Confirmation(tok))) => {
                if tok.confirmed_at.is_some() {
                    return Err(VerificationTokenError::Account(
                        AccountVerification::AccountAlreadyVerified,
                    ));
                }
                let current_time = Utc::now().naive_utc();
                if current_time - Duration::seconds(CONFIRMATION_TOKEN_EXIPIRATION_TIME)
                    > tok.created_at
                {
                    Err(VerificationTokenError::Expired)
                } else {
                    match diesel::update(
                        ct_table::confirmation_tokens.filter(ct_data::token.eq(tok.token.clone())),
                    )
                    .set(ct_data::confirmed_at.eq(Utc::now().naive_utc()))
                    .execute(&mut conn)
                    {
                        Ok(_) => {
                            match diesel::update(
                                user_table::users.filter(user_data::account_valid.eq(false)),
                            )
                            .set(user_data::account_valid.eq(true))
                            .execute(&mut conn)
                            {
                                Ok(_) => Ok("Verificated account".to_string()),
                                Err(e) => {
                                    eprintln!("Error updating token verified status: {:?}", e);
                                    Err(VerificationTokenError::ServerError(
                                        VerificationTokenServerError::DatabaseError,
                                    ))
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("Error updating token verified status: {:?}", e);
                            Err(VerificationTokenError::ServerError(
                                VerificationTokenServerError::DatabaseError,
                            ))
                        }
                    }
                }
            }
            Ok(Some(UnifiedToken::PasswordReset(tok, mail))) => {
                if tok.user_email != mail {
                    return Err(VerificationTokenError::NotFound);
                }
                let current_time = Utc::now().naive_utc();
                if current_time - Duration::seconds(CONFIRMATION_TOKEN_EXIPIRATION_TIME)
                    > tok.created_at
                {
                    return Err(VerificationTokenError::Expired);
                }
                match diesel::delete(psr_table::password_reset_tokens)
                    .filter(psr_data::token.eq(tok.token))
                    .execute(&mut conn)
                    .map_err(|e| {
                        eprintln!("Error deleting used token: {e}");
                        VerificationTokenError::ServerError(
                            VerificationTokenServerError::DatabaseError,
                        )
                    }) {
                    Ok(_) => return Ok("Token validated and consumed".to_string()),
                    Err(_) => {
                        return Err(VerificationTokenError::ServerError(
                            VerificationTokenServerError::DatabaseError,
                        ))
                    }
                };
            }
            Ok(None) => Err(VerificationTokenError::NotFound),
            Err(e) => {
                eprintln!("Database error while checking token: {:?}", e);
                Err(VerificationTokenError::ServerError(
                    VerificationTokenServerError::DatabaseError,
                ))
            }
        }
    }

    async fn send(
        _username: String,
        _u_email: String,
        _pool: DPool,
        _email_type: TokenEmailType,
        _token: Option<String>,
        _resend: bool,
        _token_type: TokenType,
    ) -> Result<String, VerificationTokenError> {
        dotenv().ok();
        let google_smtp_password = env::var("AUTH_EMAIL_PASSWORD")
            .expect("google smtp password needs to be set")
            .to_string();

        let google_smtp_name = env::var("AUTH_EMAIL_NAME")
            .expect("google smtp name needs to be set")
            .to_string();

        let token = match _token {
            Some(tok) => tok,
            None => match Self::new(_u_email.clone(), _resend, _token_type, _pool) {
                Ok(tok) => tok,
                Err(e) => {
                    eprintln!("generating token in sending erro {:?}", e);
                    return Err(VerificationTokenError::ServerError(
                        VerificationTokenServerError::TokenGenerationError,
                    ));
                }
            },
        };

        let email_body = match _email_type {
            TokenEmailType::AccountVerification => {
                email_body_generator(EmailType::AccountVerification(_username.clone(), token))
            }
            TokenEmailType::AccountVerificationResend => email_body_generator(
                EmailType::AccountVerificationResend(_username.clone(), token),
            ),
            TokenEmailType::PasswordReset => {
                email_body_generator(EmailType::ChangePassword(_username.clone(), token))
            }
            TokenEmailType::PasswordResetResend => {
                todo!()
            }
        };

        let email = lettre::Message::builder()
            .from(
                format!("Electro-Vision Auth <{}>", google_smtp_name)
                    .parse()
                    .unwrap(),
            )
            .to(format!("{} <{}>", _username, _u_email).parse().unwrap())
            .subject(match _email_type {
                TokenEmailType::AccountVerification | TokenEmailType::AccountVerificationResend => {
                    "Electro-Vision account verification"
                }
                TokenEmailType::PasswordReset | TokenEmailType::PasswordResetResend => {
                    "Electro-Vision password reset"
                }
            })
            .multipart(
                MultiPart::alternative().singlepart(SinglePart::html(email_body.to_string())),
            )
            .unwrap();

        let creds = Credentials::new(google_smtp_name, google_smtp_password);

        let mailer = lettre::SmtpTransport::relay(SMTP)
            .unwrap()
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(_) => Ok("Email sent successfully".to_string()),
            Err(e) => {
                eprintln!("{:?}", e);
                Err(VerificationTokenError::ServerError(
                    VerificationTokenServerError::EmailSendingError,
                ))
            }
        }
    }
}
