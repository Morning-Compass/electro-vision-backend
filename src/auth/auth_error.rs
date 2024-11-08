#[derive(Debug)]
pub enum VerificationTokenError {
    Expired,
    AccountAlreadyVerified,
    ServerError(VerificationTokenServerError),
    NotFound,
}

#[derive(Debug)]
pub enum VerificationTokenServerError {
    SettingExpirationDateError,
    TokenInsertionError,
    DatabaseError,
    TokenGenerationError,
    EmailSendingError,
    Other(String),
}
#[derive(Debug)]
pub enum JWTInvalid {
    Expired,
    EmailNotFound,
    ServerError,
}

#[derive(Debug)]
pub enum AuthError {
    UsernameNotFound,
    EmailNotFound,
    PasswordIncorrect,
    JWTError(JWTInvalid),
    ServerError(String),
}
