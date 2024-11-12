use crate::File;
use serde_json::Value;

pub struct ResponseHandler {
    pub json_data: String,
}

pub struct ResponseData {
    pub resp_name: ResponseValues,
}

pub struct ResponseValues {
    pub key: String,
    pub message: String,
    pub code: String,
}

pub trait Response {
    async fn file_get_contents(path_name: String) -> Result<Value, String>;
}

impl Response for ResponseHandler {
    async fn file_get_contents(path_name: String) -> Result<Value, String> {
        // Try to open the file
        let file = File::open(path_name);
        let file = match file {
            Ok(file) => file,
            Err(e) => return Err(e.to_string()),
        };

        let data: Result<Value, serde_json::Error> = serde_json::from_reader(file);
        match data {
            Ok(json_data) => Ok(json_data),
            Err(e) => Err(e.to_string()),
        }
    }
}
