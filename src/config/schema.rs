use crate::spider::ExtractionRule;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SpiderConfig {
    #[serde(default)]
    #[validate(length(min = 1))]
    pub name: String,

    #[serde(default)]
    #[validate(length(min = 1))]
    pub start_urls: Vec<String>,

    #[serde(default)]
    pub root_selector: Option<String>,

    #[serde(default)]
    pub extraction_rules: HashMap<String, ExtractionRule>,

    #[serde(default = "default_concurrency")]
    pub concurrency: usize,

    #[serde(default = "default_delay")]
    pub delay_ms: u64,

    #[serde(default)]
    pub output: Option<OutputConfig>,

    /// Optional path to a parent configuration file to inherit from
    #[serde(default)]
    pub extends: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OutputConfig {
    Console,
    Json {
        path: String,
    },
    Csv {
        path: String,
    },
    Sqlite {
        path: String,
        #[serde(default = "default_table_name")]
        table: String,
    },
}

fn default_concurrency() -> usize {
    2
}

fn default_delay() -> u64 {
    500
}

fn default_table_name() -> String {
    "scraped_data".to_string()
}
