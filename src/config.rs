use crate::error::{CliError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub business_id: Option<String>,
    #[serde(default)]
    pub token: Option<String>,
    #[serde(default)]
    pub format: Option<String>,
}

impl Config {
    pub fn config_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".arky")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load_file() -> Config {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path).unwrap_or_default();
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Config::default()
        }
    }

    pub fn save_file(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::config_path(), content)?;
        Ok(())
    }

    /// Resolve config with priority: CLI flags > env vars > config file
    pub fn resolve(
        flag_base_url: Option<&str>,
        flag_business_id: Option<&str>,
        flag_token: Option<&str>,
        flag_format: Option<&str>,
    ) -> ResolvedConfig {
        let file = Self::load_file();

        let base_url = flag_base_url
            .map(|s| s.to_string())
            .or_else(|| std::env::var("ARKY_BASE_URL").ok())
            .or(file.base_url)
            .unwrap_or_else(|| "http://localhost:3000".to_string());

        let business_id = flag_business_id
            .map(|s| s.to_string())
            .or_else(|| std::env::var("ARKY_BUSINESS_ID").ok())
            .or(file.business_id);

        let token = flag_token
            .map(|s| s.to_string())
            .or_else(|| std::env::var("ARKY_TOKEN").ok())
            .or(file.token);

        let format = flag_format
            .map(|s| s.to_string())
            .or_else(|| std::env::var("ARKY_FORMAT").ok())
            .or(file.format)
            .unwrap_or_else(|| "json".to_string());

        ResolvedConfig {
            base_url,
            business_id,
            token,
            format,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    pub base_url: String,
    pub business_id: Option<String>,
    pub token: Option<String>,
    pub format: String,
}

impl ResolvedConfig {
    pub fn require_business_id(&self) -> Result<&str> {
        self.business_id.as_deref().ok_or_else(|| {
            CliError::Config(
                "business_id required. Set via --business-id, ARKY_BUSINESS_ID, or `arky config set business_id <id>`".into(),
            )
        })
    }

    pub fn require_token(&self) -> Result<&str> {
        self.token.as_deref().ok_or_else(|| {
            CliError::Config(
                "token required. Set via --token, ARKY_TOKEN, or `arky auth login`".into(),
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let cfg = Config::default();
        assert!(cfg.base_url.is_none());
        assert!(cfg.business_id.is_none());
        assert!(cfg.token.is_none());
    }

    #[test]
    fn test_config_serde_roundtrip() {
        let cfg = Config {
            base_url: Some("http://localhost:3000".into()),
            business_id: Some("biz_123".into()),
            token: Some("tok_abc".into()),
            format: Some("json".into()),
        };
        let json = serde_json::to_string(&cfg).unwrap();
        let parsed: Config = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.base_url.unwrap(), "http://localhost:3000");
        assert_eq!(parsed.business_id.unwrap(), "biz_123");
    }

    #[test]
    fn test_resolve_defaults() {
        // Clear env vars for test isolation
        std::env::remove_var("ARKY_BASE_URL");
        std::env::remove_var("ARKY_BUSINESS_ID");
        std::env::remove_var("ARKY_TOKEN");
        std::env::remove_var("ARKY_FORMAT");

        let resolved = Config::resolve(None, None, None, None);
        assert_eq!(resolved.format, "json");
    }

    #[test]
    fn test_resolve_flag_priority() {
        std::env::set_var("ARKY_BASE_URL", "http://env-url");
        let resolved = Config::resolve(Some("http://flag-url"), None, None, None);
        assert_eq!(resolved.base_url, "http://flag-url");
        std::env::remove_var("ARKY_BASE_URL");
    }

    #[test]
    fn test_require_business_id() {
        let resolved = ResolvedConfig {
            base_url: "http://localhost".into(),
            business_id: None,
            token: None,
            format: "json".into(),
        };
        assert!(resolved.require_business_id().is_err());

        let resolved2 = ResolvedConfig {
            base_url: "http://localhost".into(),
            business_id: Some("biz_1".into()),
            token: None,
            format: "json".into(),
        };
        assert_eq!(resolved2.require_business_id().unwrap(), "biz_1");
    }
}
