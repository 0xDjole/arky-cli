use crate::client::ArkyClient;
use crate::commands::parse_data;
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum DatabaseCommand {
    /// Scan key-value data by prefix
    #[command(long_about = "Scan the key-value database by key prefix.\n\n\
        Returns all entries whose keys start with the given prefix.\n\n\
        Examples:\n\
        arky db scan users/\n\
        arky db scan config/ --limit 50\n\
        arky db scan \"\" --limit 10           # scan all keys\n\n\
        Response shape:\n\
        [{\"key\": \"users/123\", \"value\": {\"name\": \"John\"}}, ...]")]
    Scan {
        /// Key prefix to scan
        key: String,
        #[arg(long, default_value = "200")]
        limit: u32,
    },
    /// Put a key-value entry
    #[command(long_about = "Store a key-value entry in the database.\n\n\
        Value must be valid JSON. Use --old-key to rename/move a key.\n\n\
        Examples:\n\
        arky db put users/123 --value '{\"name\": \"John\", \"email\": \"john@example.com\"}'\n\
        arky db put config/theme --value '\"dark\"'\n\
        arky db put counters/visits --value '42'\n\
        arky db put users/new-key --value '{...}' --old-key users/old-key")]
    Put {
        /// Key to store under
        key: String,
        /// JSON value to store
        #[arg(long)]
        value: String,
        /// Old key to replace (for renames)
        #[arg(long)]
        old_key: Option<String>,
    },
    /// Delete a key-value entry
    #[command(long_about = "Delete a key-value entry by key.\n\n\
        Example:\n\
        arky db delete users/123")]
    Delete {
        /// Key to delete
        key: String,
    },
    /// Run a named script
    #[command(long_about = "Execute a server-side named script.\n\n\
        Scripts are pre-defined server operations. Optionally pass a value.\n\n\
        Example:\n\
        arky db run-script cleanup\n\
        arky db run-script migrate --value '{\"version\": 2}'")]
    RunScript {
        /// Script name
        name: String,
        #[arg(long)]
        value: Option<String>,
    },
}

pub async fn handle(cmd: DatabaseCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    match cmd {
        DatabaseCommand::Scan { key, limit } => {
            let params = [
                ("key", key.as_str()),
                ("limit", &limit.to_string()),
            ];
            let result = client.get("/v1/platform/data", &params).await?;
            crate::output::print_output(&result, format);
        }
        DatabaseCommand::Put {
            key,
            value,
            old_key,
        } => {
            let parsed_value = parse_data(Some(&value))?;
            let mut body = json!({ "key": key, "value": parsed_value });
            if let Some(old) = old_key {
                body["oldKey"] = json!(old);
            }
            let result = client.post("/v1/platform/data", &body).await?;
            crate::output::print_output(&result, format);
            crate::output::print_success(&format!("Stored key: {key}"));
        }
        DatabaseCommand::Delete { key } => {
            let params = [("key", key.as_str())];
            let result = client
                .delete_with_params("/v1/platform/data", &params)
                .await?;
            crate::output::print_output(&result, format);
            crate::output::print_success(&format!("Deleted key: {key}"));
        }
        DatabaseCommand::RunScript { name, value } => {
            let mut body = json!({ "name": name });
            if let Some(v) = value {
                body["value"] = json!(v);
            }
            let result = client.post("/v1/platform/scripts", &body).await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
