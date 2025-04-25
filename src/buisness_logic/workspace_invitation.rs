use crate::{
    constants::WORKSPACE_INVITATION_EXPIRATION_TIME, models_insertable::NewInvitation,
    response::Response as Res,
};
use actix_web::{post, web::Json, HttpResponse};
use chrono::NaiveDateTime;
use diesel::{prelude::Insertable, Connection, RunQueryDsl};
use serde::Deserialize;

use crate::{
    auth::find_user::{Find, FindData},
    est_conn, DPool,
};

#[derive(Deserialize)]
struct InviteToWorkspaceRequest {
    workspace_id: i32,
    invited_email: String,
    inviter_email: String,
}

#[post("/invite-to-workspace")]
pub async fn workspace_invitation(
    pool: DPool,
    req: Json<InviteToWorkspaceRequest>,
) -> HttpResponse {
    let conn = &mut est_conn(pool.clone());

    let invitation = NewInvitation {
        user_email: req.invited_email.clone(),
        token: uuid::Uuid::new_v4().to_string(),
        created_at: chrono::Utc::now().naive_utc(),
        expires_at: chrono::Utc::now().naive_utc()
            + chrono::Duration::seconds(WORKSPACE_INVITATION_EXPIRATION_TIME),
        workspace_id: req.workspace_id,
    };

    let result = conn.transaction::<_, diesel::result::Error, _>(|c| {
        diesel::insert_into(crate::schema::workspace_invitations::dsl::workspace_invitations)
            .values(&invitation)
            .execute(c)
    });

    match result {
        Ok(_) => HttpResponse::Ok().json(Res::new("Invitation created successfully")),
        Err(err) => {
            eprintln!("Error creating Invitation: {}", err);
            HttpResponse::InternalServerError()
                .json(Res::new("Server error while creating Invitation"))
        }
    }
}
