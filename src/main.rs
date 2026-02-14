use clap::{Parser, Subcommand};
use crawler::config::ConfigLoader;
use crawler::crawler::CrawlerEngine;
use crawler::metrics::snapshot::MetricsSnapshot;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[derive(Parser)]
#[command(name = "crawler")]
#[command(version = "0.1.0")]
#[command(about = "Unified Web Crawler with ChadSelect extraction", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a spider from a config file
    Run {
        /// Path to the configuration file (JSON/YAML/TOML)
        #[arg(short, long)]
        config: PathBuf,

        /// Show progress bars (stderr)
        #[arg(short, long, default_value_t = true)]
        progress: bool,
    },
    /// Validate a configuration file
    Check {
        /// Path to the configuration file
        #[arg(short, long)]
        config: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if std::env::var("RUST_LOG").is_err() {
        unsafe { std::env::set_var("RUST_LOG", "info"); }
    }
    let cli = Cli::parse();
    let logger = env_logger::Builder::from_default_env().build();
    let multi = Arc::new(indicatif::MultiProgress::new());

    match cli.command {
        Commands::Run { config, progress } => {
            if progress {
                let multi_clone = multi.clone();
                indicatif_log_bridge::LogWrapper::new((*multi_clone).clone(), logger)
                    .try_init()
                    .unwrap();
            } else {
                log::set_boxed_logger(Box::new(logger)).unwrap();
                log::set_max_level(log::LevelFilter::Info);
            }

            log::info!("Loading config from {:?}", config);
            let config_data = ConfigLoader::load(&config)?;
            log::info!("Loaded spider: {}", config_data.name);

            let spider = Arc::new(ConfigLoader::create_spider(&config_data, Some(multi.clone())).await?);
            let engine = CrawlerEngine::new(
                Duration::from_millis(config_data.delay_ms),
                config_data.concurrency,
                None,
            );

            let mut progress_bar: Option<ProgressBar> = None;
            let mut _progress_task = None;
            if progress {
                let pb = multi.add(ProgressBar::new(0));
                pb.set_style(ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
                    .progress_chars("#>-"));
                
                let mut metrics_rx = engine.watch_metrics();
                let pb_clone = pb.clone();
                progress_bar = Some(pb);
                _progress_task = Some(tokio::spawn(async move {
                    while metrics_rx.changed().await.is_ok() {
                        let snapshot: MetricsSnapshot = metrics_rx.borrow().clone();
                        pb_clone.set_length(snapshot.urls_queued);
                        pb_clone.set_position(snapshot.urls_processed);
                        pb_clone.set_message(format!(
                            "Items: {} | Success: {:.1}% | RPS: {:.2}",
                            snapshot.items_extracted,
                            snapshot.success_rate,
                            snapshot.requests_per_second
                        ));
                    }
                }));
            }

            log::info!("Starting crawl...");
            engine.run(spider).await;

            if progress {
                if let Some(task) = _progress_task {
                    task.abort();
                }
                if let Some(pb) = progress_bar {
                    let final_metrics = engine.get_metrics();
                    pb.set_style(ProgressStyle::default_bar()
                        .template("✅ [{elapsed_precise}] [{bar:40.green/blue}] {pos}/{len} {msg}")?
                        .progress_chars("#>-"));
                    pb.finish_with_message(format!(
                        "Items: {} | Success: {:.1}% | RPS: {:.2} - Completed",
                        final_metrics.items_extracted,
                        final_metrics.success_rate,
                        final_metrics.requests_per_second
                    ));
                }
            }

            let final_metrics = engine.get_metrics();
            println!("\n✅ Crawl Completed:");
            println!("   URLs Processed: {}", final_metrics.urls_processed);
            println!("   Items Extracted: {}", final_metrics.items_extracted);
            println!("   Success Rate: {:.1}%", final_metrics.success_rate);
            println!("   Average Duration: {}ms", final_metrics.avg_response_time_ms);
            println!("   Total Time: {:.1}s", final_metrics.elapsed_seconds);
        }
        Commands::Check { config } => {
            match ConfigLoader::load(&config) {
                Ok(cfg) => {
                    println!("✅ Config is valid:");
                    println!("   Name: {}", cfg.name);
                    println!("   Start URLs: {:?}", cfg.start_urls);
                    println!("   Rules: {}", cfg.extraction_rules.len());
                }
                Err(e) => {
                    eprintln!("❌ Config error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}
