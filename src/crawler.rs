use crate::spider::Spider;
use crate::metrics::collector::MetricsCollector;
use crate::metrics::snapshot::MetricsSnapshot;
use futures::stream::StreamExt;
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};
use std::time::Duration;
use tokio::sync::{mpsc, Barrier, watch, Mutex};
use tokio::time::sleep;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrawlerState {
    Idle,
    Running,
    Paused,
    Stopped,
}

pub struct CrawlerEngine {
    delay: Duration,
    concurrency: usize,
    metrics: Arc<MetricsCollector>,
    state: Arc<Mutex<CrawlerState>>,
    state_watcher: watch::Sender<CrawlerState>,
}

impl CrawlerEngine {
    pub fn new(delay: Duration, concurrency: usize, metrics: Option<Arc<MetricsCollector>>) -> Self {
        let (state_tx, _) = watch::channel(CrawlerState::Idle);

        Self {
            delay,
            concurrency,
            metrics: metrics.unwrap_or_else(|| Arc::new(MetricsCollector::new())),
            state: Arc::new(Mutex::new(CrawlerState::Idle)),
            state_watcher: state_tx,
        }
    }

    pub async fn run(&self, spider: Arc<dyn Spider>) {
        self.set_state(CrawlerState::Running).await;

        let (urls_tx, urls_rx) = mpsc::channel(1000);
        let (items_tx, items_rx) = mpsc::channel(100);

        let active_spiders = Arc::new(AtomicUsize::new(0));
        let barrier = Arc::new(Barrier::new(3)); // Main + Processor + Scraper

        // Seed initial URLs
        let initial_urls = spider.start_urls();
        let urls_tx_seed = urls_tx.clone();
        let metrics_seed = self.metrics.clone();
        tokio::spawn(async move {
            for url in initial_urls {
                let _ = urls_tx_seed.send(url).await;
                metrics_seed.increment_urls_queued();
            }
        });

        // Drop local senders in main thread
        drop(urls_tx);
        let items_tx_scraper = items_tx.clone();
        drop(items_tx);

        // 1. Processor Task
        let spider_clone = spider.clone();
        let metrics_clone = self.metrics.clone();
        let barrier_clone = barrier.clone();
        tokio::spawn(async move {
            tokio_stream::wrappers::ReceiverStream::new(items_rx)
                .for_each(|item| async {
                    metrics_clone.increment_items_processed();
                    if let Err(e) = spider_clone.process(item).await {
                        log::error!("Error processing item: {}", e);
                        metrics_clone.increment_items_failed();
                    }
                }).await;
            
            let _ = spider_clone.close().await;
            barrier_clone.wait().await;
        });

        // 2. Scraper Task
        let spider_clone = spider.clone();
        let barrier_clone = barrier.clone();
        let delay = self.delay;
        let concurrency = self.concurrency;
        let active_count = active_spiders.clone();
        let metrics_clone = self.metrics.clone();

        tokio::spawn(async move {
            let urls_stream = tokio_stream::wrappers::ReceiverStream::new(urls_rx);
            urls_stream.for_each_concurrent(concurrency, |url| {
                let spider = spider_clone.clone();
                let items_tx = items_tx_scraper.clone();
                let active = active_count.clone();
                let metrics = metrics_clone.clone();

                async move {
                    active.fetch_add(1, Ordering::SeqCst);
                    metrics.increment_active_workers();

                    let start_time = std::time::Instant::now();
                    let result = spider.scrape(url).await;
                    let duration = start_time.elapsed();

                    match result {
                        Ok((items, _new_urls)) => {
                            metrics.record_success(duration);
                            metrics.increment_urls_processed();
                            for item in items {
                                metrics.increment_items_extracted();
                                let _ = items_tx.send(item).await;
                            }
                            // In the future, we can send new_urls back to urls_tx here
                        }
                        Err(e) => {
                            metrics.record_failure(duration);
                            log::error!("Failed to scrape: {}", e);
                        }
                    }

                    sleep(delay).await;
                    active.fetch_sub(1, Ordering::SeqCst);
                    metrics.decrement_active_workers();
                }
            }).await;
            
            // CRITICAL: Drop the scraper's item sender so the processor can finish
            drop(items_tx_scraper);
            log::debug!("Scraper task finished.");
            barrier_clone.wait().await;
        });

        // 3. Main loop
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                log::info!("Shutting down...");
                self.set_state(CrawlerState::Stopped).await;
            }
            _ = barrier.wait() => {
                log::info!("Crawl finished.");
            }
        }
        
        self.set_state(CrawlerState::Stopped).await;
    }

    pub fn get_metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    pub fn watch_metrics(&self) -> watch::Receiver<MetricsSnapshot> {
        let (tx, rx) = watch::channel(self.metrics.snapshot());
        let metrics = self.metrics.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_millis(500));
            loop {
                interval.tick().await;
                if tx.send(metrics.snapshot()).is_err() {
                    break;
                }
            }
        });
        rx
    }

    pub async fn set_state(&self, state: CrawlerState) {
        let mut state_guard = self.state.lock().await;
        *state_guard = state;
        let _ = self.state_watcher.send(state);
    }
}
