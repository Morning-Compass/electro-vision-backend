use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::Error as IoError;

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseHandler {
    pub json_data: ResponseData,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseData {
    pub register_client_error: ResponseValues,
    pub register_server_error: ResponseValues,
    pub register_success: ResponseValues,
    pub account_validation_success: ResponseValues,
    pub account_validation_server_error: ResponseValues,
    pub account_validation_token_invalid: ResponseValues,
    pub email_resend_success: ResponseValues,
    pub login_username_success: ResponseValues,
    pub login_username_client_error: ResponseValues,
    pub login_username_server_error: ResponseValues,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ResponseValues {
    pub key: String,
    pub message: String,
    pub status: String,
}

#[derive(Debug)]
pub enum ResponseError {
    Io(IoError),
    Json(serde_json::Error),
}

impl From<IoError> for ResponseError {
    fn from(error: IoError) -> Self {
        ResponseError::Io(error)
    }
}

impl From<serde_json::Error> for ResponseError {
    fn from(error: serde_json::Error) -> Self {
        ResponseError::Json(error)
    }
}

pub trait ResponseTrait {
    async fn file_get_contents(path_name: String) -> Result<ResponseData, ResponseError>;
}

impl ResponseTrait for ResponseHandler {
    async fn file_get_contents(path_name: String) -> Result<ResponseData, ResponseError> {
        let file = File::open(&path_name)?;

        let response_data: ResponseData = match serde_json::from_reader(file) {
            Ok(data) => data,
            Err(e) => {
                return Err(ResponseError::Json(e));
            }
        };
        // println!("Data response {:?}", response_data);

        Ok(response_data)
    }
}
