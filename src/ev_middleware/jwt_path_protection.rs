use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::{FutureExt, LocalBoxFuture};
use std::future::{ready, Ready};

use crate::{
    auth::jwt::verify,
    constants::{NOT_PROTECTED_PATHS, PROTECTED_PATH_PREFIXES},
    response::Response as Res,
    DPool,
};

pub struct JWTPathProtection {
    pool: DPool,
}

impl JWTPathProtection {
    pub fn new(pool: DPool) -> Self {
        Self { pool }
    }
}

// Always return BoxBody
impl<S, B> Transform<S, ServiceRequest> for JWTPathProtection
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = JWTPathProtectionMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JWTPathProtectionMiddleware {
            service,
            pool: self.pool.clone(),
        }))
    }
}

pub struct JWTPathProtectionMiddleware<S> {
    service: S,
    pool: DPool,
}

impl<S, B> Service<ServiceRequest> for JWTPathProtectionMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let headers = req.headers();
        let path = req.request().uri().path().trim_end_matches('/');
        let path = if path.is_empty() { "/" } else { path };
        let method = req.method().to_string();

        let under_protected_prefix = PROTECTED_PATH_PREFIXES
            .iter()
            .any(|&prefix| path == prefix || path.starts_with(&format!("{}/", prefix)));
        let in_unprotected_list = NOT_PROTECTED_PATHS.iter().any(|&p| {
            let p = p.trim_end_matches('/');
            path == p || path.starts_with(&format!("{}/", p))
        });
        let is_protected = under_protected_prefix && !in_unprotected_list;

        println!(
            "========\npath: {}\nmethod: {}\nis_protected: {}",
            path, method, is_protected
        );

        if is_protected {
            let auth_error = headers
                .get("Authorization")
                .and_then(|hv| hv.to_str().ok())
                .and_then(|auth| {
                    if auth.starts_with("Bearer ") {
                        let token = auth[7..].trim();
                        if token.is_empty() {
                            Some("Authentication token is empty")
                        } else if !verify(token, self.pool.clone()) {
                            Some("Invalid or expired authentication token")
                        } else {
                            None // Valid token
                        }
                    } else {
                        Some("Authentication must use Bearer token format")
                    }
                })
                .or_else(|| Some("Authorization header is required"));

            if let Some(error_msg) = auth_error {
                eprintln!("Path protected, no token provided");
                let response = Res::new(error_msg);
                let http_response = HttpResponse::Unauthorized().json(response);

                let response = req.into_response(http_response);
                let fut = async move { Ok(response) };
                return Box::pin(fut);
            }
        }

        let inner_fut = self.service.call(req);
        Box::pin(inner_fut.map(|result| result.map(|res| res.map_into_boxed_body())))
    }
}
