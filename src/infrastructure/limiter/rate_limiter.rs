use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use dashmap::DashMap;
use parking_lot::Mutex;
use tokio::time::sleep;

/// A token bucket which allows fractional tokens for precise refill
#[derive(Debug)]
pub struct TokenBucket {
    capacity: f64,
    tokens: f64,
    refill_per_sec: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(capacity: f64, refill_per_sec: f64) -> Self {
        let now = Instant::now();
        Self {
            capacity,
            tokens: capacity,
            refill_per_sec,
            last_refill: now,
        }
    }

    /// Refill tokens based on elapsed time. Uses double precision arithmetic.
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        if elapsed > 0.0 {
            self.tokens = (self.tokens + elapsed * self.refill_per_sec).min(self.capacity);
            self.last_refill = now;
        }
    }

    /// Try to consume `amount` tokens. Return true if allowed, and remaining tokens.
    /// Small epsilon to avoid fp surprises
    fn try_consume(&mut self, amount: f64) -> bool {
        self.refill();
        if self.tokens + 1e-12 >= amount {
            self.tokens -= amount;
            true
        } else {
            false
        }
    }

    /// How many tokens remaining (useful for headers)
    fn remaining(&self) -> f64 {
        self.tokens
    }
}

/// Sliding window
#[derive(Debug)]
pub struct SlidingWindow {
    window_size: Duration,
    limit: u64,
    current_window_start: Instant,
    current_count: u64,
    prev_count: u64,
}

impl SlidingWindow {
    fn new(window_size: Duration, limit: u64) -> Self {
        Self {
            window_size,
            limit,
            current_window_start: Instant::now(),
            current_count: 0,
            prev_count: 0,
        }
    }

    /// Returns (allowed, effective_count)
    fn allow(&mut self) -> (bool, f64) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.current_window_start);

        if elapsed >= self.window_size {
            self.prev_count = self.current_count;
            self.current_count = 0;
            self.current_window_start = now;
        }

        let weight = elapsed.as_secs_f64() / self.window_size.as_secs_f64();
        let effective = (self.prev_count as f64) * (1.0 - weight) + (self.current_count as f64);

        if effective < self.limit as f64 {
            self.current_count += 1;
            (true, effective + 1.0)
        } else {
            (false, effective)
        }
    }
}

#[derive(Debug)]
struct RateHybridLimiter {
    bucket: TokenBucket,
    window: SlidingWindow,
    last_seen: Instant,
    per_second_limit: u64,
}

impl RateHybridLimiter {
    fn new(capacity: f64, refill_per_sec: f64, window_size: Duration, limit: u64) -> Self {
        Self {
            bucket: TokenBucket::new(capacity, refill_per_sec),
            window: SlidingWindow::new(window_size, limit),
            last_seen: Instant::now(),
            per_second_limit: limit,
        }
    }

    /// Returns (allowed, remaining_estimate, retry_after_secs)
    fn is_allowed(&mut self) -> (bool, f64, Option<u64>) {
        self.last_seen = Instant::now();
        // Try token bucket first
        if self.bucket.try_consume(1.0) {
            return (true, self.bucket.remaining(), None);
        }
        // Fallback to sliding window
        let (allow, _eff) = self.window.allow();
        if allow {
            return (true, 0.0_f64.max(self.bucket.remaining()), None);
        }
        // Rejected: compute retry-after estimate (seconds until 1 token refill or until window moves)
        let tokens_needed = 1.0 - self.bucket.remaining();
        let retry_after = if tokens_needed > 0.0 {
            // seconds to wait for next token
            ((tokens_needed / self.bucket.refill_per_sec).ceil() as u64).max(1)
        } else {
            1
        };
        (false, self.bucket.remaining(), Some(retry_after))
    }

    fn remaining_limit(&self) -> u64 {
        self.per_second_limit
    }
}

/// --- Rate limiter store & eviction ---
type Key = String;
#[derive(Clone)]
pub struct RateHybridLimiterStore {
    map: Arc<DashMap<Key, Arc<Mutex<RateHybridLimiter>>>>,
    default_capacity: f64,
    default_refill_per_sec: f64,
    default_window_size: Duration,
    default_limit: u64,
    bucket_ttl: Duration,
}

impl RateHybridLimiterStore {
    pub fn new(
        capacity: f64,
        refill_per_sec: f64,
        window_size: Duration,
        limit: u64,
        bucket_ttl: Duration,
    ) -> Self {
        let store = Self {
            map: Arc::new(DashMap::new()),
            default_capacity: capacity,
            default_refill_per_sec: refill_per_sec,
            default_window_size: window_size,
            default_limit: limit,
            bucket_ttl,
        };

        // spawn eviction task
        {
            let map_clone = store.map.clone();
            let ttl = store.bucket_ttl;
            tokio::spawn(async move {
                let interval = Duration::from_secs(30);
                loop {
                    sleep(interval).await;
                    let now = Instant::now();
                    let keys_to_remove: Vec<Key> = map_clone
                        .iter()
                        .filter_map(|entry| {
                            let b = entry.value();
                            let bl = b.lock();
                            if now.duration_since(bl.last_seen) > ttl {
                                Some(entry.key().clone())
                            } else {
                                None
                            }
                        })
                        .collect();

                    for k in keys_to_remove {
                        map_clone.remove(&k);
                    }
                }
            });
        }

        store
    }

    fn get_bucket(&self, key: &str) -> Arc<Mutex<RateHybridLimiter>> {
        if let Some(existing) = self.map.get(key) {
            existing.clone()
        } else {
            let limiter = Arc::new(Mutex::new(RateHybridLimiter::new(
                self.default_capacity,
                self.default_refill_per_sec,
                self.default_window_size,
                self.default_limit,
            )));
            match self.map.entry(key.to_string()) {
                dashmap::mapref::entry::Entry::Occupied(entry) => entry.get().clone(),
                dashmap::mapref::entry::Entry::Vacant(entry) => {
                    entry.insert(limiter.clone());
                    limiter
                }
            }
        }
    }

    pub fn is_allowed(&self, key: &str) -> (bool, f64, Option<u64>, u64) {
        let bucket = self.get_bucket(key);
        let mut b = bucket.lock();
        let (allowed, remaining, retry_after) = b.is_allowed();
        (allowed, remaining, retry_after, b.remaining_limit())
    }
}
