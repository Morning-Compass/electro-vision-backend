use crate::auth::find_user::{Find, FindData};
use crate::response::Response as Res;
use crate::schema::auth_users::dsl as auth_users_table;
use crate::schema::positions as positions_data;
use crate::schema::positions::dsl as positions_table;
use crate::schema::workspace_roles as workspace_roles_data;
use crate::schema::workspace_roles::dsl as workspace_roles_table;
use crate::schema::workspace_users as workspace_users_data;
use crate::schema::workspace_users::dsl as workspace_users_table;
use crate::{schema::auth_users as auth_users_data, DBPConn};

use crate::schema::countries as countries_data; // Import countries data
use crate::schema::full_users as full_users_data; // Import full_users data
use crate::schema::phone_dial_codes as phone_dial_codes_data; // Import phone_dial_codes data
use crate::schema::users_citizenships as users_citizenships_data; // Import users_citizenships data

use crate::schema::countries::dsl as countries_table;
use crate::schema::full_users::dsl as full_users_table;
use crate::schema::phone_dial_codes::dsl as phone_dial_codes_table;
use crate::schema::users_citizenships::dsl as users_citizenships_table;

use actix_web::{
    post,
    web::{Json, Path},
    HttpResponse,
};
use chrono::NaiveDate;
use diesel::{result::Error as DieselError, ExpressionMethods};
use diesel::{
    JoinOnDsl, NullableExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl,
    SelectableHelper,
}; // Added SelectableHelper
use serde::{Deserialize, Serialize};

use crate::{est_conn, models, DPool}; // Assuming models contains the data structures for your tables

#[derive(Deserialize)]
struct WorkerOverviewRequest {
    email: String,
}

#[derive(Deserialize)]
struct ListWorkspaceUsersPath {
    id: i32,
}

#[derive(Serialize)]
pub struct FullUserResponse {
    pub id: i32,
    pub username: String,
    pub created_at: chrono::NaiveDateTime,
    pub account_valid: bool,
    pub phone: String,
    pub phone_dial_code: String,
    pub country_of_origin: String,
    pub full_name: Option<String>,
    pub education: Option<String>,
    pub birth_date: chrono::NaiveDate,
    pub account_bank_number: Option<String>,
    pub photo: Option<String>,
    pub citizenships: Vec<String>,
    pub workspace_role: String,
    pub position: Option<String>,
    // nyi sex
    // nyi insurance
}

#[derive(Deserialize)]
struct GetSingleWorkspaceUserPath {
    workspace_id: i32,
    user_id: i32, // Added user_id to the path
}

// Changed to GET, updated path to include user_id
// Changed request body to only require owner_email for verification
#[post("/workspace/{workspace_id}/users/{user_id}")]
pub async fn get_singular_workspace_user_data(
    pool: DPool,
    path: Path<GetSingleWorkspaceUserPath>,
    req: Json<WorkerOverviewRequest>,
) -> HttpResponse {
    let workspace_id = path.workspace_id;
    let target_user_id = path.user_id;
    let owner_email = req.email.clone();

    let conn = &mut est_conn(pool.clone());

    // Verify if the requesting user (owner_email) actually owns the workspace
    let workspaces =
        match <FindData as Find>::find_workspace_by_owner_email(owner_email, pool.clone()).await {
            Ok(w) => w,
            Err(_) => {
                return HttpResponse::Forbidden() // Forbidden if owner not found for workspace
                    .json(Res::new(
                        "Access denied: Not authorized for this workspace.",
                    ));
            }
        };

    // Check if the provided workspace_id belongs to the found workspaces
    match workspaces.into_iter().find(|w| w.id == workspace_id) {
        Some(_) => {}
        None => {
            return HttpResponse::Forbidden() // Forbidden if workspace ID doesn't match owner
                .json(Res::new(
                    "Access denied: Workspace not found for this owner.",
                ));
        }
    };

    // Now, get the full data for the singular user within this workspace
    match get_single_workspace_user_full_data(conn, workspace_id, target_user_id).await {
        Ok(user_data) => HttpResponse::Ok().json(Res::new(user_data)),
        Err(DieselError::NotFound) => {
            HttpResponse::NotFound().json(Res::new("User not found in this workspace."))
        }
        Err(err) => {
            eprintln!("Error fetching singular workspace user data: {}", err);
            HttpResponse::InternalServerError()
                .json(Res::new("Server error while fetching user data."))
        }
    }
}

async fn get_single_workspace_user_full_data(
    conn: &mut DBPConn,
    workspace_id: i32,
    target_user_id: i32,
) -> Result<FullUserResponse, DieselError> {
    // 1. Verify the user is part of the workspace
    let workspace_user = workspace_users_table::workspace_users
        .filter(workspace_users_data::workspace_id.eq(workspace_id))
        .filter(workspace_users_data::user_id.eq(target_user_id))
        .first::<models::WorkspaceUser>(conn)
        .optional()?;

    let workspace_user = workspace_user.ok_or(DieselError::NotFound)?; // If not in workspace, return NotFound

    // 2. Fetch AuthUser data
    let auth_user = auth_users_table::auth_users
        .filter(auth_users_data::id.eq(target_user_id))
        .first::<models::AuthUser>(conn)?; // Should always find if workspace_user was found

    // 3. Fetch FullUser data
    let full_user = full_users_table::full_users
        .filter(full_users_data::user_id.eq(target_user_id))
        .first::<models::FullUser>(conn)?;

    // 4. Fetch PhoneDialCode
    let phone_dial_code = phone_dial_codes_table::phone_dial_codes
        .filter(phone_dial_codes_data::id.eq(full_user.phonde_dial_code_id))
        .first::<models::PhoneDialCode>(conn)?;

    // 5. Fetch Country of Origin
    let country_of_origin = countries_table::countries
        .filter(countries_data::id.eq(full_user.country_of_origin_id))
        .first::<models::Country>(conn)?;

    // 6. Fetch Citizenships
    let citizenships = users_citizenships_table::users_citizenships
        .filter(users_citizenships_data::user_id.eq(target_user_id))
        .inner_join(countries_table::countries)
        .select(models::Country::as_select())
        .load::<models::Country>(conn)?;

    let citizenships_countries = citizenships
        .into_iter()
        .map(|citizenship| citizenship.name)
        .collect::<Vec<String>>();

    // 7. Fetch Workspace Role
    let workspace_role = workspace_roles_table::workspace_roles
        .filter(workspace_roles_data::id.eq(workspace_user.workspace_role_id))
        .first::<models::WorkspaceRole>(conn)?;

    // 8. Fetch Position (if exists)
    let position: Option<String> = if let Some(position_id) = workspace_user.position_id {
        positions_table::positions
            .filter(positions_data::id.eq(position_id))
            .first::<models::Position>(conn)
            .optional()? // Propagate error if query fails, but not if position is None
            .and_then(|p| p.name) // Get the name, if Position found and name is Some
    } else {
        None
    };

    // Construct the FullUserResponse
    Ok(FullUserResponse {
        id: auth_user.id,
        username: auth_user.username,
        created_at: auth_user.created_at,
        account_valid: auth_user.account_valid,
        phone: full_user.phone,
        phone_dial_code: phone_dial_code.code,
        country_of_origin: country_of_origin.name,
        full_name: full_user.title,
        education: full_user.education,
        birth_date: full_user.birth_date,
        account_bank_number: full_user.account_bank_number,
        photo: full_user.photo,
        citizenships: citizenships_countries,
        workspace_role: workspace_role.name,
        position,
    })
}
