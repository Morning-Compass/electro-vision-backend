use crate::response::Response;
use actix_web::{
    get,
    web::{self, Data},
    HttpResponse,
};
use chrono::{NaiveDateTime, Utc};
use diesel::{prelude::*, result::Error};
use serde::{Deserialize, Serialize};

use crate::{DBPConn, DBPool};

pub type Users = Response<User>;

#[derive(Queryable, Debug, Serialize, Deserialize, Selectable, Insertable)]
#[diesel(table_name = crate::schema::users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
    pub created_at: NaiveDateTime,
    pub account_valid: bool,
}

impl User {
    pub fn new(new_user: NewUser) -> Self {
        Self {
            id: 0, //dont know what to add here for db
            username: new_user.username,
            email: new_user.email,
            password: new_user.password,
            created_at: Utc::now().naive_utc(),
            account_valid: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

impl NewUser {
    pub fn to_user(&self) -> User {
        User {
            id: 0, //temp dont know how db will behawe
            username: self.username.to_string(),
            email: self.email.to_string(),
            password: self.password.to_string(),
            created_at: Utc::now().naive_utc(),
            account_valid: false,
        }
    }
}

pub struct UserRequest {
    pub new_user: Option<NewUser>,
}

impl UserRequest {
    pub fn to_user(&self) -> Option<User> {
        match &self.new_user {
            Some(new_user) => Some(User::new(new_user.clone())),
            None => None,
        }
    }
}

#[get("/users")]
pub async fn list(pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect("CONNECTION_POOL_ERROR");
    match web::block(move || list_users(50, &conn)).await {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

pub async fn list_users(amount: i64, conn: &DBPConn) -> Result<Users, Error> {
    use crate::schema::users::dsl::*;

    let users_query = match users
        .order(created_at.desc())
        .limit(amount)
        .load::<User>(conn)
    {
        Ok(usr) => usr,
        Err(_) => vec![],
    };

    Ok(Users {
        results: users_query.into_iter().collect(),
    })
}
