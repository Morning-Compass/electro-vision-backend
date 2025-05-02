use crate::auth::confirmation_token::token::Cft;
use crate::auth::confirmation_token::token::ConfirmationToken;
use crate::auth::confirmation_token::token::TokenType;
use crate::auth::find_user::Find;
use crate::auth::find_user::FindData;
use crate::constants::WORKSPACE_ROLES;
use crate::models;
use crate::models::WorkspaceInvitation;
use crate::models_insertable;
use crate::response::Response as Res;
use crate::schema::workspace_invitations as workspace_invitations_data;
use crate::schema::workspace_invitations::dsl as workspace_invitations_table;
use crate::schema::workspace_roles::dsl as workspace_roles_table;
use crate::schema::workspace_users::dsl as workspace_users_table;
use actix_web::{post, HttpResponse};
use diesel::ExpressionMethods;
use diesel::Insertable;
use diesel::OptionalExtension;
use diesel::QueryDsl;
use diesel::{Connection, RunQueryDsl};
use serde::Deserialize;

use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct Token {
    token: String,
}

#[actix_web::put("/invitation/accept/{token}")]
pub async fn add_user_to_workspace(pool: DPool, req: actix_web::web::Path<Token>) -> HttpResponse {
    let workspace_invitation = match workspace_invitations_table::workspace_invitations
        .filter(workspace_invitations_data::token.eq(req.token.clone()))
        .first::<WorkspaceInvitation>(&mut est_conn(pool.clone()))
        .optional()
    {
        Ok(Some(invitation)) => invitation,
        Ok(None) => return HttpResponse::BadRequest().json(Res::new("Token invalid")),
        Err(e) => {
            eprintln!("error while accepting invitation: {:?}", e);
            return HttpResponse::InternalServerError().json(Res::new("Something went wrong"));
        }
    };

    match <Cft as ConfirmationToken>::confirm(
        req.token.clone(),
        TokenType::WorkspaceInvitation(workspace_invitation.workspace_id),
        pool.clone(),
    )
    .await
    {
        Ok(_) => {
            let conn = &mut est_conn(pool.clone());

            let user = match <FindData as Find>::find_auth_user_by_email(
                workspace_invitation.user_email,
                pool,
            )
            .await
            {
                Ok(user) => user,
                Err(err) => {
                    eprintln!("Error finding user: {}", err);
                    return HttpResponse::InternalServerError()
                        .json(Res::new("Server error while finding user"));
                }
            };
            // dodac inserta na workspaceroles z defaultowym worker, tam bezdie mozna dodawac jesze wiecej rol
            // cos jak na discord
            let result = conn.transaction::<_, diesel::result::Error, _>(|conn| {
                let workspace_role = models_insertable::WorkspaceRole {
                    user_id: user.id,
                    name: WORKSPACE_ROLES[3].to_string(),
                };

                let inserted_role = diesel::insert_into(workspace_roles_table::workspace_roles)
                    .values(&workspace_role)
                    .get_result::<models::WorkspaceRole>(conn)?;

                let invitation = models::WorkspaceUser {
                    user_id: user.id,
                    workspace_id: workspace_invitation.workspace_id,
                    workspace_role_id: inserted_role.id,
                    plane_file_cut_name: None,
                    position_id: None,
                    checkin_time: None,
                    checkout_time: None,
                };

                diesel::insert_into(workspace_users_table::workspace_users)
                    .values(&invitation)
                    .execute(conn)?;

                Ok(())
            });

            match result {
                Ok(_) => HttpResponse::Ok().json(Res::new("invitation accepted")),
                Err(err) => {
                    eprintln!("Error accepting invitation: {}", err);
                    HttpResponse::InternalServerError()
                        .json(Res::new("Server error while accepting invitation"))
                }
            }
        }
        Err(e) => {
            eprintln!("error while accepting invitation: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(Res::new("Something went wrong, prolly wrong id"));
        }
    }
}
