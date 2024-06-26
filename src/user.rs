use crate::constants::{APPLICATION_JSON, CONNECTION_POOL_ERROR};
use crate::models::User;
use crate::response::Response;
use actix_web::{
    get,
    web::{self, Data},
    HttpResponse,
};
use chrono::Utc;
use diesel::{prelude::*, result::Error};

use crate::{DBPConn, DBPool};

pub type Users = Response<User>;

impl User {
    pub fn new(self) -> Self {
        Self {
            id: 0, //dont know what to add here for db
            username: self.username,
            email: self.email,
            password: self.password,
            created_at: Utc::now().naive_utc(),
            account_valid: false,
        }
    }
}

pub async fn list_users(amount: i64, conn: &DBPConn) -> Result<Users, Error> {
    use crate::schema::users::dsl::*;

    let users_query = match users
        .select(User::as_select())
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

#[get("/users")]
pub async fn list(pool: Data<DBPool>) -> HttpResponse {
    let conn = pool.get().expect(CONNECTION_POOL_ERROR);
    let users_listed = web::block(move || list_users(50, &conn)).await.unwrap();

    HttpResponse::Ok()
        .content_type(APPLICATION_JSON)
        .json(users_listed)
}
