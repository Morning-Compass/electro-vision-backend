use crate::{
    auth::{
        confirmation_token::token::{Cft, ConfirmationToken, TokenEmailType, TokenType},
        find_user::{Find, FindData},
    },
    response::Response as Res,
};
use actix_web::{post, web::Json, HttpResponse};
use serde::Deserialize;

use crate::DPool;

#[derive(Deserialize)]
struct InviteToWorkspaceRequest {
    workspace_id: i32,
    invited_email: String,
    inviter_email: String,
}

#[post("/invitation/create")]
pub async fn workspace_invitation(
    pool: DPool,
    req: Json<InviteToWorkspaceRequest>,
) -> HttpResponse {
    let invited =
        <FindData as Find>::exists_by_email(req.invited_email.clone(), pool.clone()).await;

    match invited {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::BadRequest()
                .json(Res::new("User you are trying to invite doesnt exist"));
        }
        Err(e) => {
            eprintln!("Error while checking if user exists: {:?}", e);
            return HttpResponse::InternalServerError()
                .json(Res::new("Server error while checking if user exists"));
        }
    }

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
