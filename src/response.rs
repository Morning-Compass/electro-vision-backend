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

pub struct JsonResponse {
    key: String,
    value: String,
}

impl JsonResponse {
    pub fn new(key: String, response: String) -> String {
        format!(r#"{}: {}"#, key, response)
    }

    pub fn read(value: &str) -> serde_json::Value {
        serde_json::from_str(value).unwrap()
    }

    pub fn stringify<T>(value: T) -> String {
        serde_json::to_string(&value).expect("couldnt stringify")
    }
}