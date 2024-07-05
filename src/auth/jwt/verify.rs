use jsonwebtoken::{errors::Error, TokenData};

use super::Claims;

pub fn verify(token: Result<TokenData<Claims>, Error>) -> bool {
    true
}
