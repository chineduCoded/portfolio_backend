use actix_web::web;

use crate::handlers::home::home;

mod auth;
mod admin;
mod users;
mod json_error;

pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(home);

    cfg.service(
        web::scope("/api/v1")
            .configure(auth::config_routes)
            .configure(admin::config_routes)
            .configure(users::config_routes)
    );

    cfg.configure(json_error::config_routes);
}