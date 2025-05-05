use std::env;

use crate::auth::find_user::{Find, FindData};
use crate::constants::{
    DOMAIN, FRONTEND_DOMAIN, JWT_EXPIRATION_TIME, SMTP, WORKSPACE_INVITATION_EXPIRATION_TIME,
};
use crate::models::WorkspaceUser;
use crate::schema::auth_users as user_data;
use crate::schema::auth_users::dsl as user_table;
use crate::schema::confirmation_tokens::dsl as ct_table;
use crate::schema::password_reset_tokens::dsl as psr_table;
use crate::schema::workspace_invitations as workspace_invitations_data;
use crate::schema::workspace_invitations::dsl as workspace_invitations_table;
use chrono::{Duration, NaiveDateTime, Utc};
use diesel::dsl::exists;
use diesel::prelude::OptionalExtension;
use diesel::query_dsl::methods::FilterDsl;
use diesel::{ExpressionMethods, RunQueryDsl};
use dotenvy::dotenv;
use lettre::message::{MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::Transport;
use schema::confirmation_tokens as ct_data;
use schema::password_reset_tokens as psr_data;
use schema::workspace_users as workspace_users_data;
use schema::workspace_users::dsl as workspace_users_table;

use crate::auth::auth_error::{
    AccountVerification, InvitationError, VerificationTokenError, VerificationTokenServerError,
};
use crate::emails::{email_body_generator, EmailType};
use crate::{constants::CONFIRMATION_TOKEN_EXIPIRATION_TIME, est_conn, DPool};
use crate::{models, schema};

pub enum TokenEmailType {
    AccountVerification,
    AccountVerificationResend,
    PasswordReset,
    PasswordResetResend,
    WorkspaceInvitation,
}

pub enum TokenType {
    AccountVerification,
    PasswordReset(String),    //str email
    WorkspaceInvitation(i32), //i32 workspace_id
}

pub struct Cft {
    pub user_email: String,
    pub token: String,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

pub trait ConfirmationToken {
    async fn new(
        email: String,
        resend: bool,
        token_type: TokenType,
        pool: DPool,
    ) -> Result<String, VerificationTokenError>;
    async fn confirm(
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
    async fn new(
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
            TokenType::PasswordReset(_) => {}
            TokenType::WorkspaceInvitation(_) => {}
        }

        if token_exists {
            return Err(VerificationTokenError::TokenAlreadyExists);
        }

        let ctoken = Cft {
            user_email: u_email.clone(),
            token: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now().naive_utc(),
            expires_at: Utc::now().naive_utc() + chrono::Duration::seconds(JWT_EXPIRATION_TIME),
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
            TokenType::WorkspaceInvitation(workspace_id) => {
                match diesel::insert_into(workspace_invitations_table::workspace_invitations)
                    .values((
                        workspace_invitations_data::user_email.eq(ctoken.user_email),
                        workspace_invitations_data::token.eq(ctoken.token),
                        workspace_invitations_data::created_at.eq(ctoken.created_at),
                        workspace_invitations_data::expires_at.eq(ctoken.expires_at),
                        workspace_invitations_data::workspace_id.eq(workspace_id),
                    ))
                    .execute(&mut est_conn(pool))
                {
                    Ok(_) => Ok(token_to_return),
                    Err(e) => {
                        eprintln!("error in inviting to workspace: {:?}", e);
                        Err(VerificationTokenError::ServerError(
                            VerificationTokenServerError::WorkspaceInvitationError,
                        ))
                    }
                }
            }
        }
    }

    async fn confirm(
        _token: String,
        token_type: TokenType,
        pool: DPool,
    ) -> Result<String, VerificationTokenError> {
        let mut conn = est_conn(pool.clone());

        enum UnifiedToken {
            Confirmation(models::ConfirmationToken),
            PasswordReset(models::PasswordResetTokens, String),
            WorkspaceInvitation(models::WorkspaceInvitation),
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
            TokenType::WorkspaceInvitation(mail) => {
                workspace_invitations_table::workspace_invitations
                    .filter(workspace_invitations_data::token.eq(&_token))
                    .first::<models::WorkspaceInvitation>(&mut conn)
                    .optional()
                    .map(|opt| opt.map(|ct| UnifiedToken::WorkspaceInvitation(ct)))
            }
        };

        match db_token {
            Ok(Some(UnifiedToken::WorkspaceInvitation(tok))) => {
                // let invitations = workspace_invitations_table::workspace_invitations
                //     .filter(workspace_invitations_data::workspace_id.eq(tok.workspace_id))
                //     .load::<models::WorkspaceInvitation>(&mut conn)
                //     .map_err(|e| {
                //         eprintln!("Error loading workspace invitations: {:?}", e);
                //         VerificationTokenError::ServerError(
                //             VerificationTokenServerError::DatabaseError,
                //         )
                //     })?;

                // for inv in invitations {
                //     if inv.user_email == tok.user_email {
                //         return Err(VerificationTokenError::Invitation(
                //             InvitationError::AlreadyInWorkspace,
                //         ));
                //     }
                // }

                let user =
                    match <FindData as Find>::find_auth_user_by_email(tok.user_email, pool.clone())
                        .await
                    {
                        Ok(user) => user,
                        Err(e) => {
                            eprintln!("Error finding user: {:?}", e);
                            return Err(VerificationTokenError::ServerError(
                                VerificationTokenServerError::DatabaseError,
                            ));
                        }
                    };

                // let workspace = match workspace_table::workspaces
                //     .filter(workspace_data::id.eq(tok.workspace_id))
                //     .first::<models::Workspace>(&mut est_conn(pool.clone()))
                //     .optional()
                // {
                //     Ok(workspace) => workspace,
                //     Err(e) => {
                //         eprintln!("Error loading workspace: {:?}", e);
                //         return Err(VerificationTokenError::ServerError(
                //             VerificationTokenServerError::DatabaseError,
                //         ));
                //     }
                // };

                let workspace_users = match workspace_users_table::workspace_users
                    .filter(workspace_users_data::user_id.eq(user.id))
                    .filter(workspace_users_data::workspace_id.eq(tok.workspace_id))
                    .first::<WorkspaceUser>(&mut est_conn(pool))
                    .optional()
                {
                    Ok(workspace_users) => workspace_users,
                    Err(e) => {
                        eprintln!("Error loading workspace users: {:?}", e);
                        return Err(VerificationTokenError::ServerError(
                            VerificationTokenServerError::DatabaseError,
                        ));
                    }
                };

                if workspace_users.is_some() {
                    return Err(VerificationTokenError::Invitation(
                        InvitationError::UserAlreadyInWorkspace,
                    ));
                }

                let current_time = Utc::now().naive_utc();
                if current_time - Duration::seconds(WORKSPACE_INVITATION_EXPIRATION_TIME)
                    > tok.created_at
                {
                    return Err(VerificationTokenError::Expired);
                }

                return Ok("Invitation accepted".to_string());
            }
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
                                user_table::auth_users.filter(user_data::account_valid.eq(false)),
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
            None => match Self::new(_u_email.clone(), _resend, _token_type, _pool).await {
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
                email_body_generator(EmailType::AccountVerification(
                    _username.clone(),
                    format!("{}/auth/validate/account/{}", FRONTEND_DOMAIN, token),
                ))
            }
            TokenEmailType::AccountVerificationResend => {
                email_body_generator(EmailType::AccountVerificationResend(
                    _username.clone(),
                    format!("{}/auth/validate/account/{}", FRONTEND_DOMAIN, token),
                ))
            }
            TokenEmailType::PasswordReset => email_body_generator(EmailType::ChangePassword(
                _username.clone(),
                format!("{}/auth/reset/password/{}", FRONTEND_DOMAIN, token),
            )),
            TokenEmailType::PasswordResetResend => {
                todo!()
            }
            TokenEmailType::WorkspaceInvitation => {
                email_body_generator(EmailType::WorkspaceInvitation(
                    _username.clone(),
                    format!("{}/invitation/accept/workspace/{}", FRONTEND_DOMAIN, token),
                ))
            }
        };

        let email = lettre::Message::builder()
            .from(
                format!("Electro-Vision <{}>", google_smtp_name)
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
                TokenEmailType::WorkspaceInvitation => "Electro-Vision workspace invitation",
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
