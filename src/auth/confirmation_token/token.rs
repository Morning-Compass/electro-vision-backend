use std::env;

use actix_web::web::to;
use chrono::{Duration, NaiveDateTime, TimeDelta, Utc};
use diesel::prelude::OptionalExtension;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use dotenvy::dotenv;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::commands::Auth;
use lettre::Transport; // Import this for `optional` method

use crate::auth::{AuthError, VerificationTokenInvalid};
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
    fn new(email: String, pool: DPool) -> Result<String, AuthError>;
    fn confirm(token: String, pool: DPool) -> Result<String, AuthError>;
    async fn send(
        token: String,
        username: String,
        email: String,
        pool: DPool,
        email_type: TokenEmailType,
    ) -> Result<String, AuthError>;
}

impl ConfirmationToken for Cft {
    fn new(u_email: String, pool: DPool) -> Result<String, AuthError> {
        use crate::schema::confirmation_tokens::dsl::*;

        let ctoken = Cft {
            user_email: u_email,
            token: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: Utc::now()
                .naive_utc()
                .checked_add_signed(TimeDelta::seconds(CONFIRMATION_TOKEN_EXIPIRATION_TIME))
                .ok_or_else(|| AuthError::ServerError(String::from("Failed to check time")))?,
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
            Err(_) => Ok("Error inserting token".to_string()),
        }
    }

    fn confirm(_token: String, pool: DPool) -> Result<String, AuthError> {
        use crate::schema::confirmation_tokens::dsl::*;
        let mut conn = est_conn(pool);

        let db_token = match confirmation_tokens
            .filter(token.eq(&_token))
            .first::<models::ConfirmationToken>(&mut conn)
            .optional()
        {
            Ok(Some(tok)) => {
                if tok.confirmed_at.is_some() {
                    return Err(AuthError::VerificationTokenError(
                        VerificationTokenInvalid::AccountAlreadyVerified,
                    ));
                }
                let current_time = Utc::now().naive_utc();
                if current_time - Duration::seconds(CONFIRMATION_TOKEN_EXIPIRATION_TIME)
                    > tok.created_at
                {
                    Err(AuthError::VerificationTokenError(
                        VerificationTokenInvalid::Expired,
                    ))
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
                            Err(AuthError::ServerError(String::from(
                                "Unable to verify account",
                            )))
                        }
                    }
                }
            }
            Ok(None) => Err(AuthError::VerificationTokenError(
                VerificationTokenInvalid::NotFound,
            )),
            Err(e) => {
                eprintln!("Database error while verificating account {:?}", e);
                Err(AuthError::ServerError(String::from(
                    "Database Error while account verification",
                )))
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
                    Err(_) => Err(AuthError::ServerError(String::from(
                        "Error while setting users account verified",
                    ))),
                }
            }
            Err(e) => Err(e),
        }
    }

    async fn send(
        _token: String,
        _username: String,
        _u_email: String,
        _pool: DPool,
        _email_type: TokenEmailType,
    ) -> Result<String, AuthError> {
        dotenv().ok();
        let google_smtp_password =
            env::var("AUTH_EMAIL_PASSWORD").expect("google smtp password needs to be set");

        let google_smtp_name =
            env::var("AUTH_EMAIL_NAME").expect("google smtp name needs to be set");

        let email = lettre::Message::builder()
            .from(format!("Sender <{}>", google_smtp_name).parse().unwrap())
            .to(format!("Reciever <{}>", _u_email).parse().unwrap())
            .subject("Electrovision Account veryfiying")
            .body(email_body_generator(EmailType::AccountVerification(
                _username, _token,
            )))
            .unwrap();

        let creds = Credentials::new(google_smtp_name, google_smtp_password);

        let mailer = lettre::SmtpTransport::relay("smtp.gmail.com")
            .unwrap()
            .credentials(creds)
            .build();

        match mailer.send(&email) {
            Ok(_) => Ok("Email send successfully".to_string()),
            Err(e) => Err(AuthError::ServerError(e.to_string())),
        }
    }
}
