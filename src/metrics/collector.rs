use crate::metrics::snapshot::MetricsSnapshot;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct MetricsCollector {
    urls_queued: Arc<AtomicU64>,
    urls_processed: Arc<AtomicU64>,
    urls_pending: Arc<AtomicU64>,
    items_extracted: Arc<AtomicU64>,
    items_processed: Arc<AtomicU64>,
    items_failed: Arc<AtomicU64>,
    requests_total: Arc<AtomicU64>,
    requests_success: Arc<AtomicU64>,
    requests_failed: Arc<AtomicU64>,
    active_workers: Arc<AtomicU64>,
    total_response_time_ms: Arc<AtomicU64>,
    start_time: Arc<Instant>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self {
            urls_queued: Arc::new(AtomicU64::new(0)),
            urls_processed: Arc::new(AtomicU64::new(0)),
            urls_pending: Arc::new(AtomicU64::new(0)),
            items_extracted: Arc::new(AtomicU64::new(0)),
            items_processed: Arc::new(AtomicU64::new(0)),
            items_failed: Arc::new(AtomicU64::new(0)),
            requests_total: Arc::new(AtomicU64::new(0)),
            requests_success: Arc::new(AtomicU64::new(0)),
            requests_failed: Arc::new(AtomicU64::new(0)),
            active_workers: Arc::new(AtomicU64::new(0)),
            total_response_time_ms: Arc::new(AtomicU64::new(0)),
            start_time: Arc::new(Instant::now()),
        }
    }
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_urls_queued(&self) {
        self.urls_queued.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_urls_processed(&self) {
        self.urls_processed.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_items_extracted(&self) {
        self.items_extracted.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_items_processed(&self) {
        self.items_processed.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_items_failed(&self) {
        self.items_failed.fetch_add(1, Ordering::SeqCst);
    }

    pub fn increment_active_workers(&self) {
        self.active_workers.fetch_add(1, Ordering::SeqCst);
    }

    pub fn decrement_active_workers(&self) {
        self.active_workers.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn record_success(&self, duration: Duration) {
        self.requests_total.fetch_add(1, Ordering::SeqCst);
        self.requests_success.fetch_add(1, Ordering::SeqCst);
        self.total_response_time_ms
            .fetch_add(duration.as_millis() as u64, Ordering::SeqCst);
    }

    pub fn record_failure(&self, duration: Duration) {
        self.requests_total.fetch_add(1, Ordering::SeqCst);
        self.requests_failed.fetch_add(1, Ordering::SeqCst);
        self.total_response_time_ms
            .fetch_add(duration.as_millis() as u64, Ordering::SeqCst);
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        let total_requests = self.requests_total.load(Ordering::SeqCst);
        let success = self.requests_success.load(Ordering::SeqCst);
        let failed = self.requests_failed.load(Ordering::SeqCst);
        let total_time = self.total_response_time_ms.load(Ordering::SeqCst);

        let success_rate = if total_requests > 0 {
            (success as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };

        let avg_response_time_ms = if total_requests > 0 {
            total_time / total_requests
        } else {
            0
        };

        let elapsed = self.start_time.elapsed().as_secs_f64();

        MetricsSnapshot {
            urls_queued: self.urls_queued.load(Ordering::SeqCst),
            urls_processed: self.urls_processed.load(Ordering::SeqCst),
            urls_pending: self.urls_pending.load(Ordering::SeqCst),
            items_extracted: self.items_extracted.load(Ordering::SeqCst),
            items_processed: self.items_processed.load(Ordering::SeqCst),
            items_failed: self.items_failed.load(Ordering::SeqCst),
            requests_total: total_requests,
            requests_success: success,
            requests_failed: failed,
            active_workers: self.active_workers.load(Ordering::SeqCst),
            success_rate,
            avg_response_time_ms,
            requests_per_second: if elapsed > 0.0 {
                total_requests as f64 / elapsed
            } else {
                0.0
            },
            elapsed_seconds: elapsed,
        }
    }
}
