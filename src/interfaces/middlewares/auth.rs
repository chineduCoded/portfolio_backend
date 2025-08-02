use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage, HttpResponse,
};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use std::{rc::Rc, task::{Context, Poll}};

use crate::{entities::token::Claims, errors::AuthError, is_token_invalid, AppState, TokenCheckMode};

pub struct AuthMiddleware;

impl<S> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            let path = req.path();
            let method = req.method().as_str();

            if is_public_route(path, method) {
                return service.call(req).await;
            }

            let state = req.app_data::<web::Data<AppState>>()
                .ok_or_else(|| {
                    tracing::error!("AppState missing in middleware");
                    AuthError::MissingJwtService
                })?;
            
            let token = extract_token(&req)
                .ok_or_else(|| {
                    tracing::warn!("Missing or malformed Authorization header");
                    AuthError::MissingCredentials
                })?;
            
            if let Some(redis_pool) = &state.redis_pool {
                let blacklist_key = format!("access_deny:{}", token);
                let is_blacklisted = is_token_invalid(redis_pool, &blacklist_key, TokenCheckMode::Exists)
                .await
                .unwrap_or(false);

                if is_blacklisted {
                    return Ok(custom_error_response(req, HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "Access token is blacklisted"
                    }))));
                }
            }

            let claims = match get_valid_claims(&req) {
                Ok(claims) => claims,
                Err(AuthError::MissingCredentials) => {
                    tracing::warn!("Missing or invalid credentials");
                    return Ok(custom_error_response(req, HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "Missing or invalid credentials"
                    }))));
                }
                Err(_) => {
                    tracing::error!("Failed to decode JWT");
                    return Ok(custom_error_response(req, HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Internal server error"
                    }))));
                }
            };

            if let Err(forbidden_response) = enforce_admin_access(path, &claims) {
                return Ok(custom_error_response(req, forbidden_response));
            }

            req.extensions_mut().insert(claims);
            service.call(req).await
        })
    }
}

fn is_public_route(path: &str, method: &str) -> bool {
    if method == "OPTIONS" {
        return true;
    }
    
    matches!(
        (path, method),
        ("/", "GET") |
        ("/api/v1/auth/refresh", "POST") |
        ("/api/v1/auth/login", "POST") |
        ("/api/v1/auth/register", "POST")
    )
}

fn extract_token(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            let parts: Vec<&str> = header.split_whitespace().collect();
            if parts.len() == 2 && parts[0].eq_ignore_ascii_case("bearer") {
                Some(parts[1].to_string())
            } else {
                None
            }
        })
}

fn get_valid_claims(req: &ServiceRequest) -> Result<Claims, AuthError> {
    let state = req.app_data::<web::Data<AppState>>()
        .ok_or(AuthError::MissingJwtService)?;
    
    let token = extract_token(req).ok_or(AuthError::MissingCredentials)?;
    let decoded = state.auth_handler.token_service.decode_jwt(&token)?;
    Ok(decoded.claims)
}

fn enforce_admin_access(path: &str, claims: &Claims) -> Result<(), HttpResponse> {
    if path.starts_with("/admin") && !claims.admin {
        tracing::warn!("Admin access required for path: {}", path);
        tracing::warn!("User claims: {:?}", claims);
        return Err(
            HttpResponse::Forbidden().json(serde_json::json!({
                "error": "Admin access required"
            }))
        );
    }
    Ok(())
}

fn custom_error_response(req: ServiceRequest, res: HttpResponse) -> ServiceResponse<BoxBody> {
    req.into_response(res)
}