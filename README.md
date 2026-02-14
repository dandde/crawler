# Unified Crawler

A modern, high-performance web crawler built with Rust 2024. This project unifies advanced state management, real-time metrics, and a flexible plugin architecture for extraction and output.

## 🏗️ Crawler Design Pattern

The crawler follows an **asynchronous, message-passing architecture**:

1.  **State Management**: Uses a thread-safe `CrawlerState` (`Running`, `Paused`, `Stopped`) monitored via `tokio::sync::watch`. This allows for graceful shutdowns and external control.
2.  **Concurrency Model**: Separates the **Scraper Task** (fetching/parsing) from the **Processor Task** (output handling). Communication happens via buffered `mpsc` channels to ensure non-blocking operation.
3.  **Trait-driven Extensibility**:
    -   `Spider` Trait: Defines how to fetch and parse pages.
    -   `OutputHandler` Trait: Defines pluggable destinations for extracted data.
4.  **Metrics-driven monitoring**: A central `MetricsCollector` uses atomic counters to track URLs, items, and performance without locking overhead.

## ✨ Features

-   **extraction**: Powered by `ChadSelect`, supporting CSS (css:), XPath (xpath:), Regex (regex:), and JMESPath (json:).
-   **Configuration**: Multi-format support (JSON, YAML, TOML) with full validation.
-   **Inheritance**: Config files can inherit from others using the `extends` keyword.
-   **Outputs**: Built-in support for Console (pretty JSON), File (JSON/CSV), and SQLite databases.
-   **Progress**: Rich CLI feedback using `indicatif` with real-time RPS (Requests Per Second) and Success Rate.
-   **Modern**: Built on the **Rust 2024 edition**.

## ⚙️ Configuration System

The configuration defines the crawler behavior. You can use any of the supported formats.

### Inheritance
Use `extends` to point to a base configuration file. The child config overrides simple fields and merges `extraction_rules`.

### Example: YAML (with extraction rules)
```yaml
name: quotes-yaml
start_urls:
  - https://quotes.toscrape.com
root_selector: "css:.quote"
extraction_rules:
  text:
    selector: "css:.text"
    extract: text
  author:
    selector: "css:.author"
    extract: text
concurrency: 4
delay_ms: 1000
output:
  type: json
  path: ./assets/outputs/quotes.json
```

### Example: TOML (Inheritance)
```toml
# base.toml
name = "base-spider"
concurrency = 2
delay_ms = 500

# child.toml
extends = "base.toml"
name = "derived-spider"
start_urls = ["https://example.com"]
# ... rules ...
```

## 🚀 Usage

### Installation
```bash
git clone <repo>
cd crawler
cargo build --release
```

### Running a Crawl
```bash
# Run with progress bars
./target/release/crawler run --config configs/quotes.json

# Validate a config file
./target/release/crawler check --config configs/my_spider.yaml
```

### Configuration Formats

| Format | File Extension | Notes |
| :--- | :--- | :--- |
| **JSON** | `.json` | Standard web format |
| **YAML** | `.yaml`, `.yml` | Human-readable with lists |
| **TOML** | `.toml` | Great for hierarchical config |

## 🗺️ Future Features Plan

-   **JS Rendering**: Integration with headless browsers (Playwright/Puppeteer) for SPA scraping.
-   **Distributed Crawling**: Support for Redis-backed URL queues for cluster-based crawling.
-   **Automatic Retries**: Configurable exponential backoff for failed requests.
-   **Proxy Rotation**: Built-in support for proxy pools with automatic switching on failure.
-   **Sentry Integration**: Error reporting and monitoring for production deployments.
-   **TUI Dashboard**: A developer-focused terminal UI for deep inspection (Post-MVP).

---
Built by Antigravity
