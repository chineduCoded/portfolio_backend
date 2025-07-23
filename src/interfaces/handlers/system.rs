use actix_web::{web, get, HttpResponse, Responder};
use humantime::format_duration;
use once_cell::sync::Lazy;
use chrono::Utc;
use std::{
    time::Duration, 
    sync::{atomic::{AtomicI64, Ordering}, RwLock},
};
use sysinfo::System;
use serde::Serialize;
use crate::{constants::START_TIME, repositories::user::UserRepository, use_cases::extractors::AdminClaims, AppState};

#[derive(Serialize, Clone, Default)]
struct SystemInfo {
    os: String,
    kernel: String,
    hostname: String,
    cpu_count: usize,
    memory_total: String,
}

#[derive(Serialize, Clone, Default)]
struct HealthCheckResponse {
    status: String,
    uptime: String,
    timestamp: String,
    start_at: String,
    today_date: String,
    database: String,
    redis_status: String,
    version: String,
    memory_usage: String,
    system: SystemInfo,
}

static LAST_CHECK: AtomicI64 = AtomicI64::new(0);
static CACHED_STATUS: Lazy<RwLock<HealthCheckResponse>> = Lazy::new(|| 
    RwLock::new(HealthCheckResponse::default())
);

async fn build_health_response(state: &web::Data<AppState>) -> HealthCheckResponse {
    let now_utc = chrono::Utc::now();
    let uptime_duration = now_utc.signed_duration_since(*START_TIME);
    let human_uptime = format_duration(Duration::from_secs(uptime_duration.num_seconds() as u64));

    let mut sys = System::new_all();
    sys.refresh_all();

    let system_info = SystemInfo {
        os: System::name().unwrap_or_else(|| "Unknown".to_string()),
        kernel: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        hostname: System::host_name().unwrap_or_else(|| "Unknown".to_string()),
        cpu_count: sys.cpus().len(),
        memory_total: format!("{:.2} GB", sys.total_memory() as f64 / 1024.0 / 1024.0 / 1024.0),
    };

    let db_status = match state.auth_handler.user_repo.check_connection().await {
        Ok(_) => "OK",
        Err(_) => "Unavailable"
    };

    let redis_status = state.check_redis_health().await;
    let process = sys.process(sysinfo::get_current_pid().unwrap_or(0.into()));
    let memory_usage = process.map_or("Unknown".to_string(), |p| 
        format!("{:.2} MB", p.memory() as f64 / 1024.0 / 1024.0)
    );

    HealthCheckResponse {
        status: "healthy".to_string(),
        uptime: human_uptime.to_string(),
        timestamp: now_utc.to_rfc3339(),
        start_at: START_TIME.to_rfc3339(),
        today_date: now_utc.date_naive().to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        memory_usage,
        database: db_status.to_string(),
        redis_status: redis_status.to_string(),
        system: system_info,
    }
}

#[get("/health")]
async fn admin_health_check(
    _admin: AdminClaims,
    state: web::Data<AppState>,
) -> impl Responder {
    let now = Utc::now().timestamp();
    let last = LAST_CHECK.load(Ordering::Relaxed);

    if now - last > 5 {
        let response = build_health_response(&state).await;
        
        if let Ok(mut cache) = CACHED_STATUS.write() {
            *cache = response.clone();
            LAST_CHECK.store(now, Ordering::Relaxed);
        }
        
        HttpResponse::Ok().json(response)
    } else {
        match CACHED_STATUS.read() {
            Ok(response) => HttpResponse::Ok().json(response.clone()),
            Err(e) => {
                tracing::warn!("Health Check cache lock poisoned: {}", e);
                let response = build_health_response(&state).await;
                HttpResponse::Ok().json(response)
            }
        }
    }
}
