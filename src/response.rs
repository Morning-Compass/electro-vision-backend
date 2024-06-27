use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    pub response: T,
}

impl<T> Response<T> {
    pub fn new(response: T) -> Self {
        Self { response }
    }
}
