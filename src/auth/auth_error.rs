#[derive(Debug)]
pub enum VerificationTokenInvalid {
    Expired,
    AccountAlreadyVerified,
    ServerError,
    NotFound,
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
    VerificationTokenError(VerificationTokenInvalid),
    JWTError(JWTInvalid),
    ServerError(String),
}
