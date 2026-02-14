use super::OutputHandler;
use crate::error::Result;
use async_trait::async_trait;
use indicatif::MultiProgress;
use serde_json::Value;
use std::sync::Arc;

pub struct ConsoleOutput {
    multi: Option<Arc<MultiProgress>>,
}

impl ConsoleOutput {
    pub fn new(multi: Option<Arc<MultiProgress>>) -> Self {
        Self { multi }
    }
}

impl Default for ConsoleOutput {
    fn default() -> Self {
        Self::new(None)
    }
}

#[async_trait]
impl OutputHandler for ConsoleOutput {
    async fn write(&mut self, item: Value) -> Result<()> {
        let output = serde_json::to_string_pretty(&item).unwrap();
        
        if let Some(multi) = &self.multi {
            for line in output.lines() {
                multi.println(line).map_err(|e| crate::error::Error::Internal(e.to_string()))?;
            }
        } else {
            for line in output.lines() {
                println!("{}", line);
            }
        }
        Ok(())
    }
}
