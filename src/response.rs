use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    pub response: Vec<T>,
}

impl<T> Response<T> {
    pub fn new() -> Self {
        Self { response: vec![] }
    }
}
