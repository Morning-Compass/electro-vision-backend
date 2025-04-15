use crate::response::Response as Res;
use crate::{auth::jwt, DPool};
use actix_web::web::Json;
use actix_web::{post, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct VerifyRequest {
    token: String,
}

#[post("/verify_session")]
pub async fn verify_session(req: Json<VerifyRequest>, pool: DPool) -> HttpResponse {
    if !jwt::verify(&req.token, pool) {
        return HttpResponse::Unauthorized().json(Res::new("Token invalid"));
    }
    return HttpResponse::Ok().json(Res::new("Token Valid"));
}
