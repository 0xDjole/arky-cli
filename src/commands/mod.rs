pub mod account;
pub mod auth;
pub mod audience;
pub mod booking;
pub mod business;
pub mod config_cmd;
pub mod database;
pub mod media;
pub mod network;
pub mod node;
pub mod notification;
pub mod order;
pub mod platform;
pub mod product;
pub mod promo_code;
pub mod provider;
pub mod service;
pub mod shipping;
pub mod workflow;

use crate::error::{CliError, Result};
use serde_json::Value;
use std::io::Read;

/// Parse --data flag: inline JSON string, "-" for stdin, or @filename
pub fn parse_data(data: Option<&str>) -> Result<Value> {
    match data {
        None => Ok(Value::Object(serde_json::Map::new())),
        Some("-") => {
            let mut buf = String::new();
            std::io::stdin()
                .read_to_string(&mut buf)
                .map_err(|e| CliError::InvalidInput(format!("Failed to read stdin: {e}")))?;
            serde_json::from_str(&buf)
                .map_err(|e| CliError::InvalidInput(format!("Invalid JSON from stdin: {e}")))
        }
        Some(s) if s.starts_with('@') => {
            let path = &s[1..];
            let content = std::fs::read_to_string(path)
                .map_err(|e| CliError::InvalidInput(format!("Failed to read file {path}: {e}")))?;
            serde_json::from_str(&content)
                .map_err(|e| CliError::InvalidInput(format!("Invalid JSON in {path}: {e}")))
        }
        Some(s) => serde_json::from_str(s)
            .map_err(|e| CliError::InvalidInput(format!("Invalid JSON: {e}"))),
    }
}

/// Merge base object with data overlay
pub fn merge_data(base: &mut Value, overlay: Value) {
    if let (Value::Object(base_map), Value::Object(overlay_map)) = (base, overlay) {
        for (k, v) in overlay_map {
            base_map.insert(k, v);
        }
    }
}
