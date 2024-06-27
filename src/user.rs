use crate::constants::APPLICATION_JSON;
use crate::models::User;
use crate::response::Response;
use actix_web::{
    get,
    web::{self},
    HttpResponse,
};
use chrono::Utc;
use diesel::{prelude::*, result::Error};

use crate::{est_conn, models, DPool};

pub type Users = Response<Vec<User>>;

impl models::User {
    pub fn new(self) -> Self {
        Self {
            id: 0,
            username: self.username,
            email: self.email,
            password: self.password,
            created_at: Utc::now().naive_utc(),
            account_valid: false,
        }
    }
}

pub async fn list_users(amount: i64, pool: DPool) -> Result<Users, Error> {
    use crate::schema::users::dsl::*;

    let users_query = users
        .select(User::as_select())
        .order(created_at.desc())
        .limit(amount)
        .load::<User>(&mut est_conn(pool))
        .unwrap_or_else(|e| {
            eprintln!("Error querying users {:?}", e);
            vec![]
        });

    Ok(Users {
        response: users_query.into_iter().collect(),
    })
}

#[get("/users")]
pub async fn list(pool: DPool) -> HttpResponse {
    let users_listed = web::block(move || list_users(50, pool))
        .await
        .map_err(|e| {
            eprintln!("Failed to list users: {:?}", e);
            HttpResponse::InternalServerError().finish()
        })
        .unwrap();

    match users_listed.await {
        Ok(users) => HttpResponse::Ok()
            .content_type(APPLICATION_JSON)
            .json(users),
        Err(e) => {
            eprintln!("Failed to serialize users {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
