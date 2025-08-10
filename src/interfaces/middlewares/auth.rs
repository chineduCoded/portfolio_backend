use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    web, Error, HttpMessage
};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use std::{rc::Rc, task::{Context, Poll}};

use crate::{
    entities::token::Claims, 
    errors::AuthError, 
    is_token_invalid, 
    AppState, 
    TokenCheckMode
};

// Use a constant for the auth header to avoid string allocation
const AUTH_HEADER: &str = "Authorization";
const BEARER_PREFIX: &str = "bearer ";

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
                    AuthError::MissingAppState
                })?;

            let token = match extract_token(&req) {
                Some(token) => token,
                None => {
                    tracing::warn!("Missing or malformed Authorization header");
                    return Ok(req.error_response(AuthError::MissingCredentials));
                }
            };

            let claims = match get_valid_claims(&req) {
                Ok(claims) => claims,
                Err(e) => {
                    return Ok(req.error_response(e));
                }
            };
            
            if let Some(redis_pool) = &state.redis_pool {
                if let Err(e) = check_token_blacklist(redis_pool, &token).await {
                    return Ok(req.error_response(e));
                }
            }

            if !is_authorized(path, &claims) {
                let error = AuthError::Forbidden(format!(
                    "Admin access required. User {} is not an admin",
                    claims.sub
                ));
                tracing::warn!(
                    "Access denied - Path: {}, User ID: {}, Admin: {}",
                    path,
                    claims.sub,
                    claims.admin
                );
                return Ok(req.error_response(error));
            }

            req.extensions_mut().insert(claims);
            service.call(req).await
        })
    }
}

/// Maps AuthError variants to appropriate

fn is_public_route(path: &str, method: &str) -> bool {
    if method.eq_ignore_ascii_case("OPTIONS") {
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

pub fn is_authorized(path: &str, claims: &Claims) -> bool {
    if is_admin_route(path) {
        return claims.admin;
    }

    true
}

fn is_admin_route(path: &str) -> bool {
    path.starts_with("/admin/") || path == "/admin"
}

fn extract_token(req: &ServiceRequest) -> Option<String> {
    req.headers()
        .get(AUTH_HEADER)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.len() > BEARER_PREFIX.len() &&
                header[..BEARER_PREFIX.len()].eq_ignore_ascii_case(BEARER_PREFIX) {
                    Some(header[BEARER_PREFIX.len()..].trim().to_string())
            } else {
                None
            }
        })
}

fn get_valid_claims(
    req: &ServiceRequest,
) -> Result<Claims, AuthError> {
    let state = req.app_data::<web::Data<AppState>>()
        .ok_or_else(|| {
            tracing::error!("AppState missing in request extensions");
            AuthError::MissingAppState
        })?;

    let token = extract_token(req).ok_or_else(|| {
        tracing::warn!("Missing or malformed Authorization header");
        AuthError::MissingAuthHeader
    })?;

    state.auth_handler.token_service.decode_jwt(&token)
        .map(|data| data.claims)
        .map_err(|e| {
            tracing::error!("JWT validation failed: {}", e);
            e.into()
        })
}


async  fn check_token_blacklist(
    redis_pool: &deadpool_redis::Pool,
    token: &str
) -> Result<(), AuthError> {
    let blacklist_key = format!("access_deny:{}", token);
    match is_token_invalid(redis_pool, &blacklist_key, TokenCheckMode::Exists).await {
        Ok(true) => {
            tracing::warn!("Blacklisted token attempted access");
            Err(AuthError::TokenExpired)
        }
        Ok(false) => Ok(()),
        Err(e) => {
            tracing::error!("Redis error checking token blacklist: {}", e);
            Err(AuthError::RedisOperation(e.to_string()))
        }
    }
}