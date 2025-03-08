use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
pub struct Response<T> {
    pub response: T,
}

impl<T> Response<T> {
    pub fn new(response: T) -> Self {
        Self { response }
    }
}

pub struct JsonResponse {
    key: String,
    value: String,
}

impl JsonResponse {
    pub fn new(key: String, response: String) -> String {
        json!({ key: response }).to_string()
    }

    pub fn read(value: &str) -> serde_json::Value {
        serde_json::from_str(value).unwrap()
    }
}
