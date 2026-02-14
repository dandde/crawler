use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;

pub mod console;
pub mod json;
pub mod csv;
pub mod sqlite;

#[async_trait]
pub trait OutputHandler: Send + Sync {
    async fn write(&mut self, item: Value) -> Result<()>;
    async fn close(&mut self) -> Result<()> {
        Ok(())
    }
}
