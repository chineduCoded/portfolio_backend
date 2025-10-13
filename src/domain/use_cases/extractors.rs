use actix_web::{FromRequest, HttpRequest, HttpMessage};
use futures_util::future::{ready, Ready};
use crate::{entities::token::Claims, errors::AuthError};

/// Extractor for authenticated claims, ensuring the user is authenticated.
/// Returns 401 if the user is not authenticated.
/// Usage: Add `claims: AuthClaims` as a parameter to your handler function.
#[derive(Debug)]
pub struct AuthClaims(pub Claims);

impl FromRequest for AuthClaims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => ready(Ok(AuthClaims(claims.clone()))),
            None => ready(Err(AuthError::MissingCredentials.into())),
        }
    }
}

/// Extractor for admin claims, ensuring the user has admin privileges.
/// Returns 403 if the user is not an admin.
/// Returns 401 if the user is not authenticated.
#[derive(Debug)]
pub struct AdminClaims(pub Claims);

impl FromRequest for AdminClaims {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) if claims.admin => {
                ready(Ok(AdminClaims(claims.clone())))
            }
            Some(_) => {
                ready(Err(AuthError::Forbidden("Admin access required".into()).into()))
            }
            None => {
                ready(Err(AuthError::MissingCredentials.into()))
            }
        }
    }
}