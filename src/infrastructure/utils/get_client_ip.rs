use actix_web::HttpRequest;

/// Extract the client's IP address from the request, considering X-Forwarded-For if trusted
/// `trust_x_forwarded_for`: whether to trust the X-Forwarded-For header
pub fn get_client_ip(req: &HttpRequest, trust_x_forwarded_for: bool) -> String {
    if trust_x_forwarded_for {
        if let Some(forwarded) = req.headers().get("x-forwarded-for") {
            if let Ok(s) = forwarded.to_str() {
                return s.split(',').next().unwrap_or("").trim().to_string();
            }
        }
    }
    req.peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}