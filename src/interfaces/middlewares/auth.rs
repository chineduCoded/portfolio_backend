use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, web, HttpMessage
};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use std::{
    rc::Rc,
    task::{Poll, Context},
};

use crate::{errors::AuthError, AppState};

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
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

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Skip auth for public routes
            let path = req.path();
            let method = req.method().as_str();
            let open_paths = ["/login", "/register"];
            if open_paths.contains(&path) && method == "POST" {
                return service.call(req).await;
            }

            let state = req.app_data::<web::Data<AppState>>()
                .ok_or(AuthError::MissingJwtService)?;

            let jwt_service = &state.auth_handler.token_service;

            let token = extract_token(&req)
                .ok_or(AuthError::MissingCredentials)?;

            let claims = jwt_service.decode_jwt(&token).map_err(|e| {
                tracing::warn!("JWT decode failed: {}", e);
                e
            })?;

            req.extensions_mut().insert(claims.claims);
            service.call(req).await
        })
    }
}

fn extract_token(request: &ServiceRequest) -> Option<String> {
    request.headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .and_then(|header| {
            if header.starts_with("Bearer ") {
                Some(header[7..].to_string())
            } else {
                None
            }
        })
}