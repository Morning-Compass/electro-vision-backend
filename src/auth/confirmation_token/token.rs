use std::any::Any;
use std::env;

use chrono::{Duration, NaiveDateTime, TimeDelta, Utc};
use diesel::prelude::OptionalExtension;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use dotenvy::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use lettre::Transport;

use crate::auth::auth_error::{VerificationTokenError, VerificationTokenServerError};
use crate::emails::{email_body_generator, EmailType};
use crate::schema::users::account_valid;
use crate::{constants::CONFIRMATION_TOKEN_EXIPIRATION_TIME, est_conn, DPool};
use crate::{models, schema};

pub enum TokenEmailType {
    AccountVerification,
    AccountVerificationResend,
}

pub struct Cft {
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

pub trait ConfirmationToken {
    fn new(email: String, pool: DPool) -> Result<String, VerificationTokenError>;
    fn confirm(token: String, pool: DPool) -> Result<String, VerificationTokenError>;
    async fn send(
        username: String,
        email: String,
        pool: DPool,
        email_type: TokenEmailType,
        token: Option<String>,
    ) -> Result<String, VerificationTokenError>;
}

impl ConfirmationToken for Cft {
    fn new(u_email: String, pool: DPool) -> Result<String, VerificationTokenError> {
        use crate::schema::confirmation_tokens::dsl::*;

        let ctoken = Cft {
            user_email: u_email,
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

        match diesel::insert_into(confirmation_tokens)
            .values((
                user_email.eq(ctoken.user_email),
                token.eq(ctoken.token),
                created_at.eq(ctoken.created_at),
                expires_at.eq(ctoken.expires_at),
            ))
            .execute(&mut est_conn(pool))
        {
            Ok(_) => Ok(token_to_return),
            Err(_) => Err(VerificationTokenError::ServerError(
                VerificationTokenServerError::TokenInsertionError,
            )),
        }
    }

    fn confirm(_token: String, pool: DPool) -> Result<String, VerificationTokenError> {
        use crate::schema::confirmation_tokens::dsl::*;
        let mut conn = est_conn(pool);

        let db_token = match confirmation_tokens
            .filter(token.eq(&_token))
            .first::<models::ConfirmationToken>(&mut conn)
            .optional()
        {
            Ok(Some(tok)) => {
                if tok.confirmed_at.is_some() {
                    return Err(VerificationTokenError::AccountAlreadyVerified);
                }
                let current_time = Utc::now().naive_utc();
                if current_time - Duration::seconds(CONFIRMATION_TOKEN_EXIPIRATION_TIME)
                    > tok.created_at
                {
                    Err(VerificationTokenError::Expired)
                } else {
                    match diesel::update(
                        schema::confirmation_tokens::dsl::confirmation_tokens
                            .filter(schema::confirmation_tokens::dsl::token.eq(tok.token.clone())),
                    )
                    .set(schema::confirmation_tokens::confirmed_at.eq(Utc::now().naive_utc()))
                    .execute(&mut conn)
                    {
                        Ok(_) => Ok(tok),
                        Err(e) => {
                            eprintln!("Error updating token verified status: {:?}", e);
                            Err(VerificationTokenError::ServerError(
                                VerificationTokenServerError::DatabaseError,
                            ))
                        }
                    }
                }
            }
            Ok(None) => Err(VerificationTokenError::NotFound),
            Err(e) => {
                eprintln!("Database error while verifying account {:?}", e);
                Err(VerificationTokenError::ServerError(
                    VerificationTokenServerError::Other("Unknown".to_string()),
                ))
            }
        };

        match db_token {
            Ok(_) => {
                match diesel::update(
                    schema::users::dsl::users
                        .filter(schema::users::dsl::email.eq(db_token.unwrap().user_email)),
                )
                .set(account_valid.eq(true))
                .execute(&mut conn)
                {
                    Ok(_) => Ok("Account verified".to_string()),
                    Err(_) => Err(VerificationTokenError::ServerError(
                        VerificationTokenServerError::DatabaseError,
                    )),
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn send(
        _username: String,
        _u_email: String,
        _pool: DPool,
        _email_type: TokenEmailType,
        _token: Option<String>,
    ) -> Result<String, VerificationTokenError> {
        dotenv().ok();
        let google_smtp_password =
            env::var("AUTH_EMAIL_PASSWORD").expect("google smtp password needs to be set").to_string();

        let google_smtp_name =
            env::var("AUTH_EMAIL_NAME").expect("google smtp name needs to be set").to_string();

        let token = match _token {
            Some(tok) => tok,
            None => match Self::new(_u_email.clone(), _pool) {
                Ok(tok) => tok,
                Err(e) => {
                    return Err(VerificationTokenError::ServerError(
                        VerificationTokenServerError::TokenGenerationError,
                    ))
                }
            },
        };

        let email_body = email_body_generator(EmailType::AccountVerification(_username, token));

        let email = lettre::Message::builder()
            .from(format!("Sender <{}>", google_smtp_name).parse().unwrap())
            .to(format!("Receiver <{}>", _u_email).parse().unwrap())
            .subject("Electrovision Account Verification")
            .body(email_body)
            .unwrap();

        let creds = Credentials::new(google_smtp_name, google_smtp_password);

        let mailer = lettre::SmtpTransport::relay("smtp.gmail.com")
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
