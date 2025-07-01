use actix_web::{post, web, HttpResponse, Responder};
use crate::domain::entities::user::{NewUser, LoginUser};
use crate::AppState;

#[post("/register")]
pub async fn register(
    state: web::Data<AppState>,
    user: web::Json<NewUser>
) -> impl Responder {
    match state.auth_handler.register(user.into_inner()).await {
        Ok(user_response) => HttpResponse::Created().json(user_response),
        Err(e) => HttpResponse::BadRequest().body(e.to_string()),
    }
}

#[post("/login")]
pub async fn login(
    state: web::Data<AppState>,
    user: web::Json<LoginUser>
) -> impl Responder {
    match state.auth_handler.login(user.into_inner()).await {
        Ok(auth_response) => HttpResponse::Ok().json(auth_response), 
        Err(e) => HttpResponse::Unauthorized().body(e.to_string()),
    }
}

