use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub iat: usize,
    pub exp: usize,
    pub email: String,
}
