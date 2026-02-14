use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum BusinessCommand {
    /// Get the current business details
    #[command(long_about = "Fetch the business identified by --business-id or ARKY_BUSINESS_ID.\n\n\
        Example:\n\
        arky business get\n\n\
        Response shape:\n\
        {\"id\": \"biz_123\", \"key\": \"my-shop\", \"name\": \"My Shop\",\n\
         \"blocks\": [...], \"status\": \"active\", \"createdAt\": \"...\"}")]
    Get,
    /// List all businesses
    #[command(long_about = "List businesses accessible to the current account.\n\n\
        Examples:\n\
        arky business list\n\
        arky business list --limit 5\n\
        arky business list --query \"shop\"\n\n\
        Response shape:\n\
        {\"data\": [{\"id\": \"...\", \"key\": \"...\", \"name\": \"...\"}], \"cursor\": \"...\"}")]
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Create a new business
    #[command(long_about = "Create a new business.\n\n\
        The key must be unique and URL-safe (lowercase, hyphens).\n\n\
        Example:\n\
        arky business create my-shop --data '{\"name\": \"My Shop\"}'\n\n\
        Response: the created business object with generated ID.")]
    Create {
        /// Business key (unique identifier)
        key: String,
        /// JSON data for the business
        #[arg(long)]
        data: Option<String>,
    },
    /// Update a business
    #[command(long_about = "Update a business by ID.\n\n\
        Example:\n\
        arky business update BIZ_ID --data '{\"name\": \"New Name\"}'")]
    Update {
        /// Business ID
        id: String,
        /// JSON data to update
        #[arg(long)]
        data: Option<String>,
    },
    /// Delete a business
    Delete {
        /// Business ID
        id: String,
    },
}

pub async fn handle(cmd: BusinessCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    match cmd {
        BusinessCommand::Get => {
            let biz_id = client.require_business_id()?;
            let result = client.get(&format!("/v1/businesses/{biz_id}"), &[]).await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::List {
            query,
            limit,
            cursor,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref q) = query {
                params.push(("query", q.clone()));
            }
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client.get("/v1/businesses", &params_ref).await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Create { key, data } => {
            let mut body = json!({ "key": key });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client.post("/v1/businesses", &body).await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Update { id, data } => {
            let overlay = parse_data(data.as_deref())?;
            let mut body = json!({ "id": id });
            merge_data(&mut body, overlay);
            let result = client.put(&format!("/v1/businesses/{id}"), &body).await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Delete { id } => {
            let result = client.delete(&format!("/v1/businesses/{id}")).await?;
            crate::output::print_output(&result, format);
            crate::output::print_success("Business deleted");
        }
    }
    Ok(())
}
