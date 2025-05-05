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
use chrono::NaiveDate;
use diesel::result::Error as DieselError;
use diesel::{Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl};
use serde::Deserialize;

use crate::{
    auth::find_user::{Find, FindData},
    models, DPool,
};

#[derive(Deserialize)]
struct RegisterFullUserRequest {
    phone_number: String,
    phone_dial_code: Option<String>,
    country_of_origin: Option<String>,
    // forgor what it is, lets say mrs, ms yap yap,
    title: Option<String>,
    education: Option<String>,
    birth_date: NaiveDate,
    account_bank_number: Option<String>,
    // idk types of files
    email: String,
    photo: Option<Vec<u8>>,
    citizenships_countries_iso3: Option<Vec<String>>,
}

#[post("/user/register")]
pub async fn register_full_user(req: Json<RegisterFullUserRequest>, pool: DPool) -> HttpResponse {
    let conn = &mut est_conn(pool.clone());

    let auth_user =
        match <FindData as Find>::find_auth_user_by_email(req.email.clone(), pool.clone()).await {
            Ok(user) => user,
            Err(err) => {
                eprintln!("Err seraching for usr {:?}", err);
                return HttpResponse::InternalServerError()
                    .json(Res::new("No user found for that email"));
            }
        };

    let mut citizenship_country_ids = vec![];

    if let Some(countries) = &req.citizenships_countries_iso3 {
        for iso3_code in countries {
            match countries_table::countries
                .filter(countries_data::iso3.eq(iso3_code))
                .select(countries_data::id)
                .first::<i32>(conn)
                .optional()
            {
                Ok(Some(id)) => citizenship_country_ids.push(id),
                Ok(None) => {
                    return HttpResponse::BadRequest().json(Res::new(&format!(
                        "Invalid citizenship country: {}",
                        iso3_code
                    )));
                }
                Err(err) => {
                    eprintln!("DB error fetching citizenship country: {:?}", err);
                    return HttpResponse::InternalServerError()
                        .json(Res::new("Failed to fetch citizenship countries"));
                }
            }
        }
    }

    let country_id = if let Some(country) = &req.country_of_origin {
        match countries_table::countries
            .filter(countries_data::name.eq(country))
            .select(countries_data::id)
            .first::<i32>(conn)
            .optional()
        {
            Ok(Some(id)) => id,
            Ok(None) => {
                return HttpResponse::BadRequest().json(Res::new("Invalid country of origin"))
            }
            Err(err) => {
                eprintln!("DB error fetching country: {:?}", err);
                return HttpResponse::InternalServerError()
                    .json(Res::new("Failed to fetch country"));
            }
        }
    } else {
        return HttpResponse::BadRequest().json(Res::new("Missing country of origin"));
    };

    let dial_code_id = if let Some(code) = &req.phone_dial_code {
        match phone_dial_codes_table::phone_dial_codes
            .filter(phone_dial_codes_data::code.eq(code))
            .select(phone_dial_codes_data::id)
            .first::<i32>(conn)
            .optional()
        {
            Ok(Some(id)) => id,
            Ok(None) => {
                return HttpResponse::BadRequest().json(Res::new("Invalid phone dial code"))
            }
            Err(err) => {
                eprintln!("DB error fetching dial code: {:?}", err);
                return HttpResponse::InternalServerError()
                    .json(Res::new("Failed to fetch phone dial code"));
            }
        }
    } else {
        return HttpResponse::BadRequest().json(Res::new("Missing phone dial code"));
    };

    let user = models::FullUser {
        user_id: auth_user.id,
        phone: req.phone_number.clone(),
        phonde_dial_code_id: dial_code_id,
        country_of_origin_id: country_id,
        title: req.title.clone(),
        education: req.education.clone(),
        birth_date: req.birth_date.clone(),
        account_bank_number: req.account_bank_number.clone(),
        photo: req.photo.clone(),
    };

    let result = conn.transaction::<_, DieselError, _>(|c| {
        diesel::insert_into(full_users_table::full_users)
            .values(&user)
            .execute(c)?;

        for cid in citizenship_country_ids {
            let citizenship = models::UserCitizenship {
                user_id: auth_user.id,
                country_id: cid,
            };
            diesel::insert_into(users_citizenships_table::users_citizenships)
                .values(&citizenship)
                .execute(c)?;
        }
        Ok(())
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("User registered successfully")),
        Err(err) => {
            eprintln!("DB error registering user: {:?}", err);
            HttpResponse::InternalServerError().json(Res::new("Failed to register user"))
        }
    }
}
