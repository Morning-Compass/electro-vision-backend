use crate::{
    response::Response,
    schema::{
        confirmation_tokens::id,
        users::{account_valid, created_at, email, password, username},
    },
};
use actix_web::{get, web::Data, HttpResponse};
use chrono::Utc;
use diesel::{prelude::*, result::Error};
use serde::{Deserialize, Serialize};

use crate::{
    models::User,
    schema::{confirmation_tokens::created_at, users},
    DBPConn, DBPool,
};

pub type Users = Response<User>;

#[allow(unused)]
#[derive(Queryable, Debug, Serialize, Deserialize, Selectable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: chrono::DateTime<Utc>,
    pub account_valid: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl User {
    pub fn new(new_user: NewUser) -> Self {
        Self {
            id: 0, //dont know what to add here for db
            username: new_user.username,
            email: new_user.email,
            password: new_user.password,
            created_at: Utc::now(),
            account_valid: false,
        }
    }
    pub fn to_dbuser(&self) -> DBUser {
        DBUser {
            new_user: {
            id: 0,
            username: self.username.clone(),
            email: self.email.clone(),
            password: self.password.clone(),
            created_at: Utc::now().naive_utc(),
            account_valid: self.account_valid.clone(),}
        }
    }
}

pub struct DBUser {
    pub new_user: Option<NewUser>,
}

impl DBUser {
    pub fn to_user(&self) -> Option<User> {
        match &self.new_user {
            Some(new_user) => Some(User::new(new_user.clone())),
            None => None,
        }
    }
}

#[get("/users")]
pub async fn list(pool: Data<DBPool>, amount: i64, conn: &DBPConn) -> HttpResponse {
    HttpResponse::Ok().json("Hey")
}

pub async fn list_users(amount: i64, conn: &DBPConn) -> Result<Users, Error> {
    use crate::schema::users::dsl::*;

    let _users = match users
        .order(created_at.desc())
        .limit(amount)
        .load::<User>(conn)
    {
        Ok(usr) => usr,
        Err(_) => vec![],
    };

    Ok(Users {
        results: _users
            .into_iter()
            .map(|u| u.to_user())
            .collect::<Vec<User>>(),
    })
}
