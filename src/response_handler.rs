use serde::{Deserialize, Serialize};
use serde_json;
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

#[derive(Debug, Serialize, Deserialize)]
pub enum ResponseError {
    #[serde(
        serialize_with = "serialize_io_error",
        deserialize_with = "deserialize_io_error"
    )]
    Io(String),
    #[serde(
        serialize_with = "serialize_json_error",
        deserialize_with = "deserialize_json_error"
    )]
    Json(String),
}

impl From<IoError> for ResponseError {
    fn from(error: IoError) -> Self {
        ResponseError::Io(format!("{:?}", error))
    }
}

impl From<serde_json::Error> for ResponseError {
    fn from(error: serde_json::Error) -> Self {
        ResponseError::Json(format!("{:?}", error))
    }
}

fn serialize_io_error<S>(error: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(error)
}

fn deserialize_io_error<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer)
}

fn serialize_json_error<S>(error: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(error)
}

fn deserialize_json_error<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer)
}

pub trait ResponseTrait {
    async fn file_get_contents(path_name: String) -> Result<ResponseData, ResponseError>;
}

impl ResponseTrait for ResponseHandler {
    async fn file_get_contents(path_name: String) -> Result<ResponseData, ResponseError> {
        let file = File::open(&path_name).map_err(ResponseError::from)?;

        let response_data = serde_json::from_reader(file).map_err(ResponseError::from)?;

        Ok(response_data)
    }
}
