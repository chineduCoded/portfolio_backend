use actix_web::{web, HttpResponse, Responder};

use crate::{entities::blog_post::{NewBlogPostRequest, UpdateBlogPostRequest}, use_cases::extractors::AdminClaims, AppState};


pub async fn create_blog_post(
    _claims: AdminClaims,
    state: web::Data<AppState>,
    data: web::Json<NewBlogPostRequest>
) -> impl Responder {
    let blog_post_handler = &state.blog_handler;

    match blog_post_handler.create_blog_post(data.into_inner()).await {
        Ok(response) => HttpResponse::Ok().json(response),
        Err(e) => {
            tracing::error!("Error creating blog post: {}", e);
            HttpResponse::InternalServerError().body("Failed to create blog post")
        }
    }
}

pub async fn get_all_blog_posts(
    _claims: AdminClaims,
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let blog_post_handler = &state.blog_handler;

    let published_only = query.get("published_only").map_or(false, |v| v == "true");
    let page = query.get("page").and_then(|v| v.parse::<u32>().ok()).unwrap_or(1);
    let per_page = query.get("per_page").and_then(|v| v.parse::<u32>().ok()).unwrap_or(10);

    match blog_post_handler.get_all_blog_posts(published_only, page, per_page).await {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(e) => {
            tracing::error!("Error fetching blog posts: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch blog posts")
        }
    }
}

pub async fn get_recent_blog_posts(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let blog_post_handler = &state.blog_handler;

    let limit = query.get("limit").and_then(|v| v.parse::<u32>().ok()).unwrap_or(5);

    match blog_post_handler.get_recent_blog_posts(limit).await {
        Ok(posts) => HttpResponse::Ok().json(posts),
        Err(e) => {
            tracing::error!("Error fetching recent blog posts: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch recent blog posts")
        }
    }
}

pub async fn get_blog_post_by_id(
    _claims: AdminClaims,
    post_id: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let blog_post_handler = &state.blog_handler;

    match blog_post_handler.get_blog_post_by_id(&post_id).await {
        Ok(post) => HttpResponse::Ok().json(post),
        Err(e) => {
            tracing::error!("Error fetching blog post: {}", e);
            HttpResponse::InternalServerError().body("Failed to fetch blog post")
        }
    }
}

pub async fn update_blog_post(
    _claims: AdminClaims,
    post_id: web::Path<String>,
    state: web::Data<AppState>,
    data: web::Json<UpdateBlogPostRequest>,
) -> impl Responder {
    let blog_post_handler = &state.blog_handler;

    match blog_post_handler.update_blog_post(&post_id, &data.into_inner()).await {
        Ok(updated_post) => HttpResponse::Ok().json(updated_post),
        Err(e) => {
            tracing::error!("Error updating blog post: {}", e);
            HttpResponse::InternalServerError().body("Failed to update blog post")
        }
    }
}

pub async fn publish_blog_post(
    _claims: AdminClaims,
    post_id: web::Path<String>,
    state: web::Data<AppState>,
) -> impl Responder {
    let blog_post_handler = &state.blog_handler;

    match blog_post_handler.publish_blog_post(&post_id).await {
        Ok(published_post) => HttpResponse::Ok().json(published_post),
        Err(e) => {
            tracing::error!("Error publishing blog post: {}", e);
            HttpResponse::InternalServerError().body("Failed to publish blog post")
        }
    }
}

pub async fn delete_blog_post(
    _claims: AdminClaims,
    post_id: web::Path<String>,
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let blog_post_handler = &state.blog_handler;

    let hard_delete = query.get("hard_delete").map_or(false, |v| v == "true");

    match blog_post_handler.delete_blog_post(&post_id, hard_delete).await {
        Ok(_) => HttpResponse::Ok().body("Blog post deleted successfully"),
        Err(e) => {
            tracing::error!("Error deleting blog post: {}", e);
            HttpResponse::InternalServerError().body("Failed to delete blog post")
        }
    }
}