use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub urls_queued: u64,
    pub urls_processed: u64,
    pub urls_pending: u64,
    pub items_extracted: u64,
    pub items_processed: u64,
    pub items_failed: u64,
    pub requests_total: u64,
    pub requests_success: u64,
    pub requests_failed: u64,
    pub active_workers: u64,
    pub success_rate: f64,
    pub avg_response_time_ms: u64,
    pub requests_per_second: f64,
    pub elapsed_seconds: f64,
}
