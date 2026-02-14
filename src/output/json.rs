use super::OutputHandler;
use crate::error::Result;
use async_trait::async_trait;
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub struct JsonOutput {
    file: File,
    first: bool,
}

impl JsonOutput {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)?;

        write!(file, "[")?;

        Ok(Self {
            file,
            first: true,
        })
    }
}

#[async_trait]
impl OutputHandler for JsonOutput {
    async fn write(&mut self, item: Value) -> Result<()> {
        if !self.first {
            write!(self.file, ",")?;
        } else {
            self.first = false;
        }

        serde_json::to_writer(&mut self.file, &item)?;
        Ok(())
    }

    async fn close(&mut self) -> Result<()> {
        write!(self.file, "]")?;
        Ok(())
    }
}
