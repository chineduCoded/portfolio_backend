use actix_web::web;

use crate::handlers::{auth, system::admin_health_check};

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .service(admin_health_check)
            .service(auth::admin_dashboard)
    );
}