use crate::error::{Error, Result};
use crate::spider::GenericSpider;
use crate::output::{OutputHandler, console::ConsoleOutput, json::JsonOutput, csv::CsvOutput, sqlite::SqliteOutput};
use crate::config::schema::{SpiderConfig, OutputConfig};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use validator::Validate;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<SpiderConfig> {
        let path = path.as_ref();
        let mut visited = HashSet::new();
        Self::load_with_inheritance(path, &mut visited, false)
    }

    fn load_with_inheritance(
        path: &Path,
        visited: &mut HashSet<PathBuf>,
        is_parent_load: bool,
    ) -> Result<SpiderConfig> {
        let path = fs::canonicalize(path).map_err(|e| {
            Error::Config(format!("{}: {}", path.display(), e))
        })?;

        if visited.contains(&path) {
            return Err(Error::Config(format!(
                "Circular inheritance detected involving {}",
                path.display()
            )));
        }
        visited.insert(path.clone());

        let config = Self::load_file(&path)?;

        let final_config = if let Some(parent_path_str) = &config.extends {
            let parent_path = path.parent()
                .ok_or_else(|| Error::Config(format!(
                    "Cannot determine parent directory for {}",
                    path.display()
                )))?
                .join(parent_path_str);

            let parent_config = Self::load_with_inheritance(&parent_path, visited, true)?;
            Self::merge_configs(parent_config, config)
        } else {
            config
        };

        if !is_parent_load {
            final_config.validate()
                .map_err(|e| Error::Validation(e))?;
        }

        Ok(final_config)
    }

    fn load_file(path: &Path) -> Result<SpiderConfig> {
        let content = fs::read_to_string(path)
            .map_err(|e| Error::Config(format!("{}: {}", path.display(), e)))?;

        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => {
                let config: SpiderConfig = serde_json::from_str(&content)?;
                Ok(config)
            }
            Some("yaml") | Some("yml") => {
                let config: SpiderConfig = serde_yaml::from_str(&content)?;
                Ok(config)
            }
            Some("toml") => {
                let config: SpiderConfig = toml::from_str(&content)?;
                Ok(config)
            }
            _ => Err(Error::Config(format!(
                "Unsupported file extension: {}",
                path.display()
            ))),
        }
    }

    fn merge_configs(mut parent: SpiderConfig, child: SpiderConfig) -> SpiderConfig {
        if !child.name.is_empty() {
            parent.name = child.name;
        }
        if !child.start_urls.is_empty() {
            parent.start_urls = child.start_urls;
        }
        if child.root_selector.is_some() {
            parent.root_selector = child.root_selector;
        }
        if child.concurrency != 2 {
            parent.concurrency = child.concurrency;
        }
        if child.delay_ms != 500 {
            parent.delay_ms = child.delay_ms;
        }
        if child.output.is_some() {
            parent.output = child.output;
        }

        for (key, rule) in child.extraction_rules {
            parent.extraction_rules.insert(key, rule);
        }

        parent.extends = None;
        parent
    }

    pub async fn create_spider(
        config: &SpiderConfig,
        multi: Option<Arc<indicatif::MultiProgress>>,
    ) -> Result<GenericSpider> {
        let handler: Box<dyn OutputHandler> = if let Some(out_config) = &config.output {
            match out_config {
                OutputConfig::Console => Box::new(ConsoleOutput::new(multi)),
                OutputConfig::Json { path } => Box::new(JsonOutput::new(PathBuf::from(path))?),
                OutputConfig::Csv { path } => Box::new(CsvOutput::new(PathBuf::from(path))?),
                OutputConfig::Sqlite { path, table } => {
                    Box::new(SqliteOutput::new(PathBuf::from(path), table.clone()).await?)
                }
            }
        } else {
            Box::new(ConsoleOutput::new(multi))
        };

        Ok(GenericSpider::new(
            config.name.clone(),
            config.start_urls.clone(),
            config.root_selector.clone(),
            config.extraction_rules.clone(),
            handler,
        ))
    }
}
