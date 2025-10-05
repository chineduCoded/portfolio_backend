use actix_web::{web, HttpResponse, Responder};

use crate::{entities::blog_post::{NewBlogPostRequest, UpdateBlogPostRequest}, errors::AppError, use_cases::extractors::AdminClaims, AppState};


pub async fn create_blog_post(
    _claims: AdminClaims,
    state: web::Data<AppState>,
    data: web::Json<NewBlogPostRequest>
) -> Result<impl Responder, AppError> {
    let blog_post_handler = &state.blog_handler;

    let response = blog_post_handler
        .create_blog_post(data.into_inner())
        .await?;

    Ok(HttpResponse::Created().json(response))
}

pub async fn get_all_blog_posts(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, AppError> {
    let blog_post_handler = &state.blog_handler;

    let page = query.get("page").and_then(|v| v.parse::<u32>().ok()).unwrap_or(1);
    let per_page = query.get("per_page")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(10)
        .min(100);

    let posts = blog_post_handler
        .get_all_blog_posts(true, page, per_page)
        .await?;

    Ok(HttpResponse::Ok().json(posts))
}

pub async fn get_recent_blog_posts(
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, AppError> {
    let blog_post_handler = &state.blog_handler;

    let limit = query.get("limit")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(5)
        .min(50);


    let posts = blog_post_handler.get_recent_blog_posts(limit, true).await?;

    Ok(HttpResponse::Ok().json(posts))
}

pub async fn get_blog_post_by_id(
    post_id: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let blog_post_handler = &state.blog_handler;

    let post = blog_post_handler.get_blog_post_by_id(&post_id).await?;
    Ok(HttpResponse::Ok().json(post))
}


pub async fn update_blog_post(
    _claims: AdminClaims,
    post_id: web::Path<String>,
    state: web::Data<AppState>,
    data: web::Json<UpdateBlogPostRequest>,
) -> Result<impl Responder, AppError> {
    let blog_post_handler = &state.blog_handler;
    let updated_post = blog_post_handler.update_blog_post(&post_id, &data.into_inner()).await?;
    Ok(HttpResponse::Ok().json(updated_post))
}

pub async fn publish_blog_post(
    _claims: AdminClaims,
    post_id: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<impl Responder, AppError> {
    let blog_post_handler = &state.blog_handler;
    let published_post = blog_post_handler.publish_blog_post(&post_id).await?;
    Ok(HttpResponse::Ok().json(published_post))
}

pub async fn delete_blog_post(
    _claims: AdminClaims,
    post_id: web::Path<String>,
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, AppError> {
    let blog_post_handler = &state.blog_handler;
    let hard_delete = query.get("hard_delete").map_or(false, |v| v == "true");
    blog_post_handler.delete_blog_post(&post_id, hard_delete).await?;
    Ok(HttpResponse::NoContent().finish())
}

// Additional handlers for the admin interface can be added here
// such as listing all posts including unpublished ones, etc.
pub async fn admin_get_all_blog_posts(
    _claims: AdminClaims,
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, AppError> {
    let blog_post_handler = &state.blog_handler;

    let page = query.get("page").and_then(|v| v.parse::<u32>().ok()).unwrap_or(1);
    let per_page = query.get("per_page")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(10)
        .min(100);

    let posts = blog_post_handler
        .get_all_blog_posts(false, page, per_page)
        .await?;

    Ok(HttpResponse::Ok().json(posts))
}

pub async fn admin_get_recent_blog_posts(
    _claims: AdminClaims,
    state: web::Data<AppState>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, AppError> {
    let blog_post_handler = &state.blog_handler;

    let limit = query.get("limit")
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(5)
        .min(50);
    
    let posts = blog_post_handler.get_recent_blog_posts(limit, false).await?;
    Ok(HttpResponse::Ok().json(posts))
}
