use crate::config::{Config, ResolvedConfig};
use crate::error::{CliError, Result};
use crate::output::Format;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    /// Show the current resolved configuration
    #[command(long_about = "Display the current configuration with resolved values.\n\n\
        Shows values from all sources (CLI flags > env vars > config file).\n\
        Token is partially masked for security.\n\n\
        Example:\n\
        arky config show")]
    Show,
    /// Set a config value (base_url, business_id, token, format)
    #[command(long_about = "Persist a configuration value to ~/.arky/config.json.\n\n\
        Valid keys:\n\
          base_url      Server URL (e.g., http://localhost:8000)\n\
          business_id   Default business ID for all commands\n\
          token         Auth token (usually set via `arky auth verify`)\n\
          format        Default output format: json, table, plain\n\n\
        Examples:\n\
        arky config set base_url http://localhost:8000\n\
        arky config set business_id 0bbf0256-2fe9-4517-81ff-ebf8ebb2f373\n\
        arky config set format table")]
    Set {
        /// Config key to set
        key: String,
        /// Value to set
        value: String,
    },
    /// Show the config file path
    #[command(long_about = "Print the path to the config file.\n\n\
        Default: ~/.arky/config.json\n\n\
        Example:\n\
        arky config path")]
    Path,
}

pub async fn handle(cmd: ConfigCommand, resolved: &ResolvedConfig, format: &Format) -> Result<()> {
    match cmd {
        ConfigCommand::Show => {
            let display = serde_json::json!({
                "base_url": resolved.base_url,
                "business_id": resolved.business_id,
                "token": resolved.token.as_ref().map(|t| {
                    if t.len() > 20 {
                        format!("{}...{}", &t[..10], &t[t.len()-6..])
                    } else {
                        t.clone()
                    }
                }),
                "format": resolved.format,
                "config_file": Config::config_path().to_string_lossy().to_string(),
            });
            crate::output::print_output(&display, format);
        }
        ConfigCommand::Set { key, value } => {
            let mut cfg = Config::load_file();
            match key.as_str() {
                "base_url" | "base-url" => cfg.base_url = Some(value),
                "business_id" | "business-id" => cfg.business_id = Some(value),
                "token" => cfg.token = Some(value),
                "format" => cfg.format = Some(value),
                _ => {
                    return Err(CliError::InvalidInput(format!(
                        "Unknown config key: {key}. Valid keys: base_url, business_id, token, format"
                    )));
                }
            }
            cfg.save_file()?;
            crate::output::print_success(&format!("Config '{key}' saved"));
        }
        ConfigCommand::Path => {
            println!("{}", Config::config_path().to_string_lossy());
        }
    }
    Ok(())
}
