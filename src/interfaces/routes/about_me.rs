use actix_web::web;

use crate::handlers::about_me;


pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/about")
            .service(
                web::resource("/introduction")
                    .route(web::post().to(about_me::create_about_me))
            )
    );
}