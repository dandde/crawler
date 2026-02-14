use crate::error::{Error, Result};
use crate::output::OutputHandler;
use async_trait::async_trait;
use chadselect::ChadSelect;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExtractionType {
    Text,
    Attribute(String),
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionRule {
    pub selector: String,
    pub extract: ExtractionType,
}

#[async_trait]
pub trait Spider: Send + Sync {
    fn name(&self) -> String;
    fn start_urls(&self) -> Vec<String>;
    async fn scrape(&self, url: String) -> Result<(Vec<Value>, Vec<String>)>;
    async fn process(&self, item: Value) -> Result<()>;
    async fn close(&self) -> Result<()> {
        Ok(())
    }
}

pub struct GenericSpider {
    pub name: String,
    pub start_urls: Vec<String>,
    pub client: Client,
    pub root_selector: Option<String>,
    pub extraction_rules: HashMap<String, ExtractionRule>,
    pub output_handler: Arc<Mutex<Box<dyn OutputHandler>>>,
}

impl GenericSpider {
    pub fn new(
        name: String,
        start_urls: Vec<String>,
        root_selector: Option<String>,
        extraction_rules: HashMap<String, ExtractionRule>,
        output_handler: Box<dyn OutputHandler>,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(10))
            .user_agent("Unified-Crawler/1.0")
            .build()
            .expect("Building HTTP client");

        Self {
            name,
            start_urls,
            client,
            root_selector,
            extraction_rules,
            output_handler: Arc::new(Mutex::new(output_handler)),
        }
    }

    fn extract_data(&self, cs: &ChadSelect, doc_index: i32) -> Result<Value> {
        let mut item = serde_json::Map::new();
        let mut found_data = false;

        for (field_name, rule) in &self.extraction_rules {
            // ChadSelect select returns a String
            // We might need to handle different extraction types if ChadSelect supports them directly,
            // but for now we follow the rule.extract.
            
            // NOTE: ChadSelect's select(index, query) might need the prefix (css:, xpath:, regex:)
            // We assume rule.selector already has it or we could add a default.
            let query = if rule.selector.contains(':') {
                rule.selector.clone()
            } else {
                format!("css:{}", rule.selector)
            };

            let val = cs.select(doc_index, &query);
            
            if !val.is_empty() {
                item.insert(field_name.clone(), json!(val));
                found_data = true;
            }
        }

        if found_data {
            Ok(Value::Object(item))
        } else {
            Err(Error::Extraction("No data found for item".to_string()))
        }
    }
}

#[async_trait]
impl Spider for GenericSpider {
    fn name(&self) -> String {
        self.name.clone()
    }

    fn start_urls(&self) -> Vec<String> {
        self.start_urls.clone()
    }

    async fn scrape(&self, url: String) -> Result<(Vec<Value>, Vec<String>)> {
        log::info!("Visiting: {}", url);

        let res = self.client.get(&url).send().await?;
        let status = res.status();
        if !status.is_success() {
            return Err(Error::Internal(format!("HTTP error: {}", status)));
        }
        
        let html = res.text().await?;
        log::debug!("HTML length: {} bytes", html.len());
        
        let mut cs = ChadSelect::new();
        cs.add_html(html);
        
        let mut items = Vec::new();
        
        if let Some(root) = &self.root_selector {
            let root_query = if root.contains(':') {
                root.clone()
            } else {
                format!("css:{}", root)
            };
            
            log::debug!("Processing with root selector: {}", root_query);
            
            let roots = cs.query(-1, &root_query);
            log::debug!("Root selector '{}' found {} matches", root_query, roots.len());

            let mut field_results = HashMap::new();
            let mut max_len = 0;

            for (field_name, rule) in &self.extraction_rules {
                let rule_selector = rule.selector.split_once(':').map(|s| s.1).unwrap_or(&rule.selector);
                
                // Try combined selector first: root + space + rule
                let full_query = format!("{} {}", root_query, rule_selector);
                let results = cs.query(-1, &full_query);
                
                log::debug!("Field '{}' with query '{}' found {} results", field_name, full_query, results.len());
                
                max_len = max_len.max(results.len());
                field_results.insert(field_name.clone(), results);
            }
            
            if max_len == 0 && !roots.is_empty() {
                log::warn!("Found {} roots but 0 items. Checking if rules should be absolute...", roots.len());
            }
            
            log::info!("Extracted {} items from {}", max_len, url);

            for i in 0..max_len {
                let mut item = serde_json::Map::new();
                for (field_name, results) in &field_results {
                    if let Some(val) = results.get(i) {
                        item.insert(field_name.clone(), json!(val));
                    }
                }
                if !item.is_empty() {
                    items.push(Value::Object(item));
                }
            }
        } else {
            // Single item mode
            if let Ok(item) = self.extract_data(&cs, 0) {
                items.push(item);
            }
        }

        Ok((items, vec![]))
    }

    async fn process(&self, item: Value) -> Result<()> {
        let mut handler = self.output_handler.lock().await;
        handler.write(item).await
    }

    async fn close(&self) -> Result<()> {
        let mut handler = self.output_handler.lock().await;
        handler.close().await
    }
}
