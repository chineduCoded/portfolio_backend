use actix_web::{web, Error, HttpResponse, Responder};

use crate::{entities::contact_me::NewContactMeForm, AppState};


const EMAIL_LIMIT: u32 = 2;
const EMAIL_WINDOW_SECS: usize = 3600;

pub async fn create_contact_me(
    state: web::Data<AppState>,
    form: web::Json<NewContactMeForm>,
) -> Result<impl Responder, Error> {

    // Normalize to lower_case and URL-encode to keep the Redis key safe
    let email_norm = form.email.trim().to_lowercase();
    let email_enc = urlencoding::encode(&email_norm);

    let email_key = format!("rl:email:{}", email_enc);
    let email_cnt = state.redis_incr_with_ttl(&email_key, EMAIL_WINDOW_SECS).await?;
    
    if email_cnt > EMAIL_LIMIT {
        return Ok(HttpResponse::TooManyRequests().json(serde_json::json!({
            "error": "Too many messages from this email address. Please try again later."
        })));
    }

    let response = state.contact_handler
        .create_contact_message(form.into_inner()).await?;

    Ok(HttpResponse::Created().json(response))
}