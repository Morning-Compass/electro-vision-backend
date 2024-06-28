use crate::models::User;
use crate::user::NoIdUser;
use crate::{est_conn, response, DPool};
use actix_web::web;
use actix_web::{post, web::Json, HttpResponse};
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error;
use serde_derive::Deserialize;

struct RequestLoginUsername {
    username: String,
    password: String,
}

pub async fn login_username() -> HttpResponse {}
