#[derive(Debug)]
pub enum VerificationTokenError {
    Account(AccountVerification),
    Password(PasswordReset),
    ServerError(VerificationTokenServerError),
    TokenAlreadyExists,
    Expired,
    NotFound,
    Invitation(InvitationError),
}

#[derive(Debug)]
pub enum InvitationError {
    AlreadyInWorkspace,
    NotInvited,
    Expired,
    NotFound,
}

#[derive(Debug)]
pub enum AccountVerification {
    AccountAlreadyVerified,
}

#[derive(Debug)]
pub enum PasswordReset {}

#[derive(Debug)]
pub enum VerificationTokenServerError {
    SettingExpirationDateError,
    TokenInsertionError,
    DatabaseError,
    TokenGenerationError,
    EmailSendingError,
    Other(String),
    WorkspaceInvitationError,
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
