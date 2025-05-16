// after 13 may
//
//
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

#[derive(Deserialize)]
struct UpdateFullUserRequest {
    phone_number: Option<String>,
    phone_dial_code: Option<String>,
    country_of_origin: Option<String>,
    title: Option<String>,
    education: Option<String>,
    birth_date: Option<NaiveDate>,
    account_bank_number: Option<String>,
    photo: Option<Vec<u8>>,
    citizenships_countries_iso3: Option<Vec<String>>,
    email: String,
}

#[put("/user/update")]
pub async fn update_full_user(req: Json<UpdateFullUserRequest>, pool: DPool) -> HttpResponse {
    let conn = &mut est_conn(pool.clone());

    let auth_user =
        match <FindData as Find>::find_auth_user_by_email(req.email.clone(), pool.clone()).await {
            Ok(u) => u,
            Err(e) => match e {
                DieselError::NotFound => {
                    return HttpResponse::Unauthorized().json(Res::new("User not found"));
                }
                _ => {
                    eprintln!("error while checking for user in update{:?} ", e);
                    return HttpResponse::InternalServerError()
                        .json(Res::new("Unknown error occurred"));
                }
            },
        };

    let mut dial_code_id = None;
    if let Some(code) = &req.phone_dial_code {
        match phone_dial_codes_table::phone_dial_codes
            .filter(phone_dial_codes_data::code.eq(code))
            .select(phone_dial_codes_data::id)
            .first::<i32>(conn)
            .optional()
        {
            Ok(Some(id)) => dial_code_id = Some(id),
            Ok(None) => {
                return HttpResponse::BadRequest().json(Res::new("Invalid phone dial code"));
            }
            Err(err) => {
                eprintln!("DB error fetching dial code: {:?}", err);
                return HttpResponse::InternalServerError()
                    .json(Res::new("Failed to fetch phone dial code"));
            }
        }
    }

    let mut country_id = None;
    if let Some(country) = &req.country_of_origin {
        match countries_table::countries
            .filter(countries_data::name.eq(country))
            .select(countries_data::id)
            .first::<i32>(conn)
            .optional()
        {
            Ok(Some(id)) => country_id = Some(id),
            Ok(None) => {
                return HttpResponse::BadRequest().json(Res::new("Invalid country of origin"));
            }
            Err(err) => {
                eprintln!("DB error fetching country: {:?}", err);
                return HttpResponse::InternalServerError()
                    .json(Res::new("Failed to fetch country"));
            }
        }
    }

    let result = conn.transaction::<_, DieselError, _>(|c| {
        // Update main user data
        diesel::update(
            full_users_table::full_users.filter(full_users_data::user_id.eq(auth_user.id)),
        )
        .set((
            req.phone_number
                .as_ref()
                .map(|p| full_users_data::phone.eq(p)),
            dial_code_id.map(|d| full_users_data::phonde_dial_code_id.eq(d)),
            country_id.map(|c| full_users_data::country_of_origin_id.eq(c)),
            req.title.as_ref().map(|t| full_users_data::title.eq(t)),
            req.education
                .as_ref()
                .map(|e| full_users_data::education.eq(e)),
            req.birth_date
                .as_ref()
                .map(|b| full_users_data::birth_date.eq(b)),
            req.account_bank_number
                .as_ref()
                .map(|a| full_users_data::account_bank_number.eq(a)),
            req.photo.as_ref().map(|p| full_users_data::photo.eq(p)),
        ))
        .execute(c)?;

        if let Some(countries) = &req.citizenships_countries_iso3 {
            diesel::delete(
                users_citizenships_table::users_citizenships
                    .filter(users_citizenships_data::user_id.eq(auth_user.id)),
            )
            .execute(c)?;

            for iso3_code in countries {
                let country_id = countries_table::countries
                    .filter(countries_data::iso3.eq(iso3_code))
                    .select(countries_data::id)
                    .first::<i32>(c)?;

                let citizenship = models::UserCitizenship {
                    user_id: auth_user.id,
                    country_id,
                };
                diesel::insert_into(users_citizenships_table::users_citizenships)
                    .values(&citizenship)
                    .execute(c)?;
            }
        }

        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("User updated successfully")),
        Err(err) => {
            eprintln!("DB error updating user: {:?}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to update user"))
        }
    }
}
