use actix_web::http::StatusCode;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};
use crate::domain::entities::user::LoginUser;
use crate::entities::token::{AuthResponse, RefreshTokenRequest};
use crate::entities::user::{LogoutRequest, NewUser};
use crate::handlers::json_error::{handle_auth_handler_error, json_error};
use crate::use_cases::extractors::AdminClaims;
use crate::AppState;

#[post("/register")]
pub async fn register(
    state: web::Data<AppState>,
    user: web::Json<NewUser>
) -> impl Responder {
    match state.auth_handler.register(user.into_inner()).await {
        Ok(response) => HttpResponse::Created().json(response),
        Err(e) => e.to_http_response(),
    }
}

#[post("/login")]
pub async fn login(
    state: web::Data<AppState>,
    user: web::Json<LoginUser>
) -> impl Responder {
    match state.auth_handler.login(user.into_inner()).await {
        Ok(auth_response) => HttpResponse::Ok().json(auth_response), 
        Err(e) => handle_auth_handler_error(e),
    }
}

#[post("/refresh-token")]
pub async fn refresh_token(
    state: web::Data<AppState>,
    request: web::Json<RefreshTokenRequest>,
) -> impl Responder {
    match state.auth_handler.refresh_token(&request.refresh_token, &state).await {
        Ok(auth_response) => HttpResponse::Ok().json(AuthResponse {
            access_token: auth_response.access_token,
            refresh_token: auth_response.refresh_token,
            token_type: "Bearer".to_string(),
        }),
        Err(e) => handle_auth_handler_error(e),
    }
}


#[post("/logout")]
pub async fn logout(
    request: HttpRequest,
    state: web::Data<AppState>,
    body: web::Json<LogoutRequest>
) -> impl Responder {
    let access_token = match state.auth_handler.extract_token(&request) {
        Some(token) => token,
        None => {
            return json_error(
                StatusCode::UNAUTHORIZED,
                "Unauthorized",
                "Missing access token in Authorization header"
            )
        }
    };


    match state.auth_handler.logout(&body.refresh_token, &access_token, &state).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"message": "Logged out successfully"})),
        Err(e) => handle_auth_handler_error(e),
    }
}

#[get("/dashboard")]
pub async fn admin_dashboard(
    admin: AdminClaims,
    _state: web::Data<AppState>
) -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "message": format!("Welcome, admin {}", admin.0.sub)
    }))
}
