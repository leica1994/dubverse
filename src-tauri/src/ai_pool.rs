#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, OwnedSemaphorePermit, RwLock, Semaphore};

// ── TokenBucket ──────────────────────────────────────────────────────────────

struct TokenBucket {
    tokens: f64,
    capacity: f64,
    refill_rate: f64, // tokens per second (0 = unlimited)
    last_update: Instant,
}

impl TokenBucket {
    fn new(rate_per_min: u32) -> Self {
        let (capacity, refill_rate) = if rate_per_min == 0 {
            (f64::MAX, 0.0)
        } else {
            let rps = rate_per_min as f64 / 60.0;
            (rate_per_min as f64, rps)
        };
        TokenBucket {
            tokens: capacity,
            capacity,
            refill_rate,
            last_update: Instant::now(),
        }
    }

    /// Returns Ok(()) if a token is available, or Err(wait_ms) if we need to wait.
    fn try_acquire(&mut self) -> Result<(), u64> {
        if self.refill_rate == 0.0 {
            return Ok(());
        }
        let elapsed = self.last_update.elapsed().as_secs_f64();
        self.last_update = Instant::now();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.capacity);
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            Ok(())
        } else {
            let wait_secs = (1.0 - self.tokens) / self.refill_rate;
            Err((wait_secs * 1000.0).ceil() as u64)
        }
    }
}

// ── ConfigController ─────────────────────────────────────────────────────────

pub(crate) struct ConfigController {
    semaphore: RwLock<Arc<Semaphore>>,
    rate_limiter: Mutex<TokenBucket>,
}

impl ConfigController {
    fn new(concurrent_limit: u32, rate_limit: u32) -> Self {
        ConfigController {
            semaphore: RwLock::new(Arc::new(Semaphore::new(concurrent_limit as usize))),
            rate_limiter: Mutex::new(TokenBucket::new(rate_limit)),
        }
    }

    /// Acquire rate-limit + semaphore slot, polling every 100ms while checking abort.
    async fn acquire(
        &self,
        abort_flag: &Arc<AtomicBool>,
    ) -> Result<OwnedSemaphorePermit, String> {
        // 1. Rate limit
        loop {
            if abort_flag.load(Ordering::Relaxed) {
                return Err("已取消".to_string());
            }
            let wait_ms = {
                let mut bucket = self.rate_limiter.lock().await;
                match bucket.try_acquire() {
                    Ok(()) => break,
                    Err(ms) => ms,
                }
            };
            // Sleep in 100ms chunks so we can check abort
            let chunks = (wait_ms / 100).max(1);
            for _ in 0..chunks {
                if abort_flag.load(Ordering::Relaxed) {
                    return Err("已取消".to_string());
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        // 2. Semaphore — poll with 10ms interval, 300s timeout
        let deadline = Instant::now() + Duration::from_secs(300);
        loop {
            if abort_flag.load(Ordering::Relaxed) {
                return Err("已取消".to_string());
            }
            if Instant::now() >= deadline {
                return Err("等待并发槽超时（300s）".to_string());
            }
            let sem = {
                let guard = self.semaphore.read().await;
                Arc::clone(&*guard)
            };
            match sem.try_acquire_owned() {
                Ok(permit) => {
                    return Ok(permit);
                }
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
        }
    }

    /// Replace semaphore with a new one (in-flight permits keep the old semaphore alive).
    async fn update_semaphore(&self, concurrent_limit: u32) {
        let mut guard = self.semaphore.write().await;
        *guard = Arc::new(Semaphore::new(concurrent_limit as usize));
    }

    /// Replace rate limiter.
    async fn update_rate_limit(&self, rate_per_min: u32) {
        let mut guard = self.rate_limiter.lock().await;
        *guard = TokenBucket::new(rate_per_min);
    }
}

// ── AiPoolManager ─────────────────────────────────────────────────────────────

/// Per-config timeout stored alongside client.
struct CachedClient {
    client: reqwest::Client,
    timeout_secs: u64,
}

pub struct AiPoolManager {
    controllers: Mutex<HashMap<String, Arc<ConfigController>>>,
    clients: Mutex<HashMap<String, CachedClient>>,
}

impl AiPoolManager {
    pub fn new() -> Self {
        AiPoolManager {
            controllers: Mutex::new(HashMap::new()),
            clients: Mutex::new(HashMap::new()),
        }
    }

    /// Get or create a controller for the given config id.
    pub async fn ensure_controller(
        &self,
        id: &str,
        concurrent_limit: u32,
        rate_limit: u32,
    ) -> Arc<ConfigController> {
        let mut map = self.controllers.lock().await;
        map.entry(id.to_string())
            .or_insert_with(|| Arc::new(ConfigController::new(concurrent_limit, rate_limit)))
            .clone()
    }

    /// Get (or build) the cached HTTP client for this config id.
    /// Recreates if timeout changed.
    pub async fn get_or_create_client(&self, id: &str, timeout_secs: u64) -> reqwest::Client {
        let mut map = self.clients.lock().await;
        if let Some(cached) = map.get(id) {
            if cached.timeout_secs == timeout_secs {
                return cached.client.clone();
            }
        }
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_default();
        map.insert(
            id.to_string(),
            CachedClient {
                client: client.clone(),
                timeout_secs,
            },
        );
        client
    }

    /// Update controller limits in-place (running tasks keep old semaphore).
    pub async fn update_controller(&self, id: &str, concurrent_limit: u32, rate_limit: u32) {
        let map = self.controllers.lock().await;
        if let Some(ctrl) = map.get(id) {
            ctrl.update_semaphore(concurrent_limit).await;
            ctrl.update_rate_limit(rate_limit).await;
        }
    }

    /// Force client recreation on next request (by removing the cached entry).
    pub async fn update_timeout(&self, id: &str) {
        let mut map = self.clients.lock().await;
        map.remove(id);
    }

    /// Remove controller and client for a deleted config.
    pub async fn remove(&self, id: &str) {
        self.controllers.lock().await.remove(id);
        self.clients.lock().await.remove(id);
    }

    /// Acquire semaphore permit for `id`, creating controller on first use.
    pub async fn acquire(
        &self,
        id: &str,
        concurrent_limit: u32,
        rate_limit: u32,
        abort_flag: &Arc<AtomicBool>,
    ) -> Result<OwnedSemaphorePermit, String> {
        let ctrl = self.ensure_controller(id, concurrent_limit, rate_limit).await;
        ctrl.acquire(abort_flag).await
    }
}
