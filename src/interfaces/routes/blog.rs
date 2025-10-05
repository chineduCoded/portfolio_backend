use actix_web::web;

use crate::handlers::blog_posts;

pub fn config_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/blog/posts")
            .service(
                web::resource("")
                    .route(web::post().to(blog_posts::create_blog_post))
                    .route(web::get().to(blog_posts::get_all_blog_posts))
            )
            .service(
                web::resource("/recent/{limit}")
                    .route(web::get().to(blog_posts::get_recent_blog_posts))
            )
            .service(
                web::resource("/{post_id}")
                    .route(web::get().to(blog_posts::get_blog_post_by_id))
                    .route(web::patch().to(blog_posts::update_blog_post))
                    .route(web::delete().to(blog_posts::delete_blog_post))
            )
            .service(
                web::resource("/{post_id}/publish")
                    .route(web::post().to(blog_posts::publish_blog_post))
            )
    );
}