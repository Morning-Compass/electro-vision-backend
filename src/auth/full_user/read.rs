use crate::schema::auth_users::dsl as auth_users_table;
use crate::schema::countries::dsl as countries_table;
use crate::schema::full_users::dsl as full_users_table;
use crate::schema::phone_dial_codes::dsl as phone_dial_codes_table;
use crate::schema::users_citizenships::dsl as users_citizenships_table;

use crate::schema::auth_users as auth_users_data;
use crate::schema::countries as countries_data;
use crate::schema::full_users as full_users_data;
use crate::schema::phone_dial_codes as phone_dial_codes_data;
use crate::schema::users_citizenships as users_citizenships_data;

use crate::{est_conn, response::Response as Res};
use crate::{models, DPool};
use actix_web::post;
use actix_web::{web::Json, HttpResponse};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct FullUserResponse {
    id: i32,
    username: String,
    created_at: chrono::NaiveDateTime,
    account_valid: bool,
    phone: String,
    phone_dial_code: String,
    country_of_origin: String,
    title: Option<String>,
    education: Option<String>,
    birth_date: chrono::NaiveDate,
    account_bank_number: Option<String>,
    photo: Option<String>,
    citizenships: Vec<String>,
}

#[derive(Deserialize)]
struct GetFullUserRequest {
    email: String,
}

#[post("/user/list")]
pub async fn get_full_user(req: Json<GetFullUserRequest>, pool: DPool) -> HttpResponse {
    let conn = &mut est_conn(pool);

    let auth_user = match auth_users_table::auth_users
        .filter(auth_users_data::email.eq(&req.email))
        .first::<models::AuthUser>(conn)
        .optional()
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::NotFound().json(Res::new("User not found"));
        }
        Err(err) => {
            eprintln!("Error fetching auth user: {}", err);
            return HttpResponse::InternalServerError().json(Res::new("Error fetching user"));
        }
    };

    let full_user = match full_users_table::full_users
        .filter(full_users_data::user_id.eq(auth_user.id))
        .first::<models::FullUser>(conn)
        .optional()
    {
        Ok(Some(user)) => user,
        Ok(None) => {
            return HttpResponse::NotFound().json(Res::new("Full user details not found"));
        }
        Err(err) => {
            eprintln!("Error fetching full user: {}", err);
            return HttpResponse::InternalServerError()
                .json(Res::new("Error fetching user details"));
        }
    };

    let phone_dial_code = match phone_dial_codes_table::phone_dial_codes
        .filter(phone_dial_codes_data::id.eq(full_user.phonde_dial_code_id))
        .first::<models::PhoneDialCode>(conn)
    {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error fetching phone dial code: {}", err);
            return HttpResponse::InternalServerError().json(Res::new("Error fetching phone code"));
        }
    };

    let country_of_origin = match countries_table::countries
        .filter(countries_data::id.eq(full_user.country_of_origin_id))
        .first::<models::Country>(conn)
    {
        Ok(country) => country,
        Err(err) => {
            eprintln!("Error fetching country: {}", err);
            return HttpResponse::InternalServerError().json(Res::new("Error fetching country"));
        }
    };

    let citizenships = match users_citizenships_table::users_citizenships
        .filter(users_citizenships_data::user_id.eq(auth_user.id))
        .inner_join(countries_table::countries)
        .select(models::Country::as_select())
        .load::<models::Country>(conn)
    {
        Ok(countries) => countries,
        Err(err) => {
            eprintln!("Error fetching citizenships: {}", err);
            return HttpResponse::InternalServerError()
                .json(Res::new("Error fetching citizenships"));
        }
    };

    let citizenships_countries = citizenships
        .iter()
        .map(|citizenship| citizenship.name.clone())
        .collect::<Vec<String>>();

    let response = FullUserResponse {
        id: auth_user.id,
        username: auth_user.username,
        created_at: auth_user.created_at,
        account_valid: auth_user.account_valid,
        phone: full_user.phone,
        phone_dial_code: phone_dial_code.code,
        country_of_origin: country_of_origin.name,
        title: full_user.title,
        education: full_user.education,
        birth_date: full_user.birth_date,
        account_bank_number: full_user.account_bank_number,
        photo: full_user.photo,
        citizenships: citizenships_countries,
    };

    HttpResponse::Ok().json(Res::new(response))
}
