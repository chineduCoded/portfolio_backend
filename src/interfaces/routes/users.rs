use actix_web::web;
use crate::handlers::users;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/users")
            .service(
                web::resource("/me")
                    .route(web::get().to(users::me))
            )
            .service(
                web::resource("/{user_id}")
                    .route(web::get().to(users::get_user))
                    .route(web::delete().to(users::delete_user))
            )
    );
}