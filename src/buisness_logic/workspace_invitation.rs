use crate::{
    auth::{
        confirmation_token::token::{Cft, ConfirmationToken, TokenEmailType, TokenType},
        find_user::{Find, FindData},
    },
    response::Response as Res,
};
use actix_web::{post, web::Json, HttpResponse};
use diesel::{prelude::Insertable, Connection, RunQueryDsl};
use serde::Deserialize;

use crate::{est_conn, DPool};

#[derive(Deserialize)]
struct InviteToWorkspaceRequest {
    workspace_id: i32,
    invited_email: String,
    inviter_email: String,
}

#[post("/invite_to_workspace")]
pub async fn workspace_invitation(
    pool: DPool,
    req: Json<InviteToWorkspaceRequest>,
) -> HttpResponse {
    let result = <Cft as ConfirmationToken>::send(
        "Worker".to_string(),
        req.invited_email.clone(),
        pool.clone(),
        TokenEmailType::WorkspaceInvitation,
        None,
        false,
        TokenType::WorkspaceInvitation(req.workspace_id.clone()),
    )
    .await;

    let workspace =
        <FindData as Find>::find_workspace_by_owner_email(req.inviter_email.clone(), pool).await;

    // add errorhndling to find, custom class
    match workspace {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Erroer while inviting: {:?}", e);
            return HttpResponse::BadRequest().json(Res::new("Couldnt invite to workspace due to server error. Are you sure that you are owner of workspace?"));
        }
    }

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Invitation created successfully")),
        Err(_) => HttpResponse::InternalServerError()
            .json(Res::new("Server error while creating Invitation")),
    }
}
