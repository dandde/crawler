use super::OutputHandler;
use crate::error::{Error, Result};
use async_trait::async_trait;
use serde_json::Value;
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;

pub struct SqliteOutput {
    pool: SqlitePool,
    table_name: String,
    initialized: bool,
}

impl SqliteOutput {
    pub async fn new(path: PathBuf, table_name: String) -> Result<Self> {
        let conn_str = format!("sqlite:{}?mode=rwc", path.display());
        let pool = SqlitePool::connect(&conn_str).await
            .map_err(|e| Error::Database(e))?;
            
        Ok(Self {
            pool,
            table_name,
            initialized: false,
        })
    }

    async fn ensure_table(&mut self, item: &serde_json::Map<String, Value>) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        let mut columns = Vec::new();
        for key in item.keys() {
            columns.push(format!("{} TEXT", key));
        }
        
        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY, {})",
            self.table_name,
            columns.join(", ")
        );
        
        sqlx::query(&query).execute(&self.pool).await
            .map_err(|e| Error::Database(e))?;
            
        self.initialized = true;
        Ok(())
    }
}

#[async_trait]
impl OutputHandler for SqliteOutput {
    async fn write(&mut self, item: Value) -> Result<()> {
        if let Value::Object(map) = item {
            self.ensure_table(&map).await?;
            
            let keys: Vec<_> = map.keys().map(|k| k.as_str()).collect();
            let placeholders: Vec<_> = (1..=keys.len()).map(|i| format!("?{}", i)).collect();
            
            let query = format!(
                "INSERT INTO {} ({}) VALUES ({})",
                self.table_name,
                keys.join(", "),
                placeholders.join(", ")
            );
            
            let mut q = sqlx::query(&query);
            for key in keys {
                let val = match map.get(key).unwrap() {
                    Value::String(s) => s.clone(),
                    v => v.to_string(),
                };
                q = q.bind(val);
            }
            
            q.execute(&self.pool).await
                .map_err(|e| Error::Database(e))?;
        }
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        self.pool.close().await;
        Ok(())
    }
}
