use chrono::{DateTime, Utc};
use once_cell::sync::Lazy;

pub static START_TIME: Lazy<DateTime<Utc>> = Lazy::new(Utc::now);