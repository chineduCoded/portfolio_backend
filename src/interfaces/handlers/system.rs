use actix_web::{get, HttpResponse, Responder};
use humantime::format_duration;
use std::time::Duration;

use crate::constants::START_TIME;

#[get("/health")]
async fn health_check() -> impl Responder {
    let now_utc = chrono::Utc::now();
    let uptime_duration = now_utc.signed_duration_since(*START_TIME);
    let human_uptime = format_duration(Duration::from_secs(uptime_duration.num_seconds() as u64));

    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "uptime": human_uptime.to_string(),
        "timestamp": now_utc.to_rfc3339(),
        "start_at": START_TIME.to_rfc3339(),
        "Today's date": now_utc.date_naive(),
    }))
}
