use actix_web::web;

use crate::handlers::about_me;


pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/about-me")
            .service(
                web::resource("")
                    .route(web::post().to(about_me::create_about_me))
            )
            .service(
                web::resource("/introduction")
                    .route(web::get().to(about_me::get_about_me))
            )
            .service(
                web::resource("/{about_me_id}")
                    .route(web::delete().to(about_me::delete_about_me))
            )
    );
}