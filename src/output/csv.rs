use super::OutputHandler;
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::path::PathBuf;

pub struct CsvOutput {
    writer: csv::Writer<std::fs::File>,
    headers_written: bool,
}

impl CsvOutput {
    pub fn new(path: PathBuf) -> Result<Self> {
        let writer = csv::Writer::from_path(path)
            .map_err(|e| crate::error::Error::Internal(e.to_string()))?;
            
        Ok(Self {
            writer,
            headers_written: false,
        })
    }
}

#[async_trait]
impl OutputHandler for CsvOutput {
    async fn write(&mut self, item: Value) -> Result<()> {
        if let Value::Object(map) = item {
            if !self.headers_written {
                let headers: Vec<_> = map.keys().collect();
                self.writer.write_record(headers)
                    .map_err(|e| crate::error::Error::Internal(e.to_string()))?;
                self.headers_written = true;
            }
            
            let values: Vec<_> = map.values().map(|v| match v {
                Value::String(s) => s.clone(),
                _ => v.to_string(),
            }).collect();
            
            self.writer.write_record(values)
                .map_err(|e| crate::error::Error::Internal(e.to_string()))?;
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.writer.flush()
            .map_err(|e| crate::error::Error::Internal(e.to_string()))?;
        Ok(())
    }
}
