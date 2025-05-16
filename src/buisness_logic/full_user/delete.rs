use crate::schema::countries as countries_data;
use crate::schema::countries::dsl as countries_table;
use crate::schema::full_users as full_users_data;
use crate::schema::full_users::dsl as full_users_table;
use crate::schema::phone_dial_codes as phone_dial_codes_data;
use crate::schema::phone_dial_codes::dsl as phone_dial_codes_table;
use crate::schema::users_citizenships as users_citizenships_data;
use crate::schema::users_citizenships::dsl as users_citizenships_table;

use crate::{est_conn, response::Response as Res};
use actix_web::{post, web::Json, HttpResponse};
use actix_web::{put, web};
use chrono::NaiveDate;
use diesel::result::Error as DieselError;
use diesel::{Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use serde::Deserialize;

use crate::{
    auth::find_user::{Find, FindData},
    models, DPool,
};
// after 13 may
//
//
//
#[actix_web::delete("/user/delete/{user_id}")]
pub async fn delete_full_user(user_id: actix_web::web::Path<i32>, pool: DPool) -> HttpResponse {
    let conn = &mut est_conn(pool);

    let result = conn.transaction::<_, DieselError, _>(|c| {
        diesel::delete(
            users_citizenships_table::users_citizenships
                .filter(users_citizenships_data::user_id.eq(*user_id)),
        )
        .execute(c)?;

        diesel::delete(full_users_table::full_users.filter(full_users_data::user_id.eq(*user_id)))
            .execute(c)?;

        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("User deleted successfully")),
        Err(err) => {
            eprintln!("DB error deleting user: {:?}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to delete user"))
        }
    }
}
