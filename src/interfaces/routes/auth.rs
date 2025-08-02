use actix_web::web;

use crate::handlers::auth;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .service(auth::register)
            .service(auth::login)
            .service(auth::refresh_token)
            .service(auth::logout)
    );
}