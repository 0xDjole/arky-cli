use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum AudienceCommand {
    /// Get an audience by ID
    #[command(long_about = "Fetch a single audience.\n\n\
        Example:\n\
        arky audience get AUDIENCE_ID\n\n\
        Response shape:\n\
        {\"id\": \"...\", \"key\": \"premium-members\", \"name\": \"Premium Members\",\n\
         \"blocks\": [...], \"subscriberCount\": 42}")]
    Get {
        /// Audience ID
        id: String,
    },
    /// List audiences
    #[command(long_about = "List audiences (access groups and subscription tiers).\n\n\
        Examples:\n\
        arky audience list\n\
        arky audience list --query \"premium\"")]
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Create an audience (access group with optional subscription pricing)
    #[command(long_about = "Create an audience for access control and subscriptions.\n\n\
        Audiences can be used as:\n\
          - Access groups: gate content behind membership\n\
          - Subscription tiers: charge recurring fees for access\n\
          - Mailing lists: manage subscribers for newsletters\n\n\
        Blocks: same as nodes (text, localized_text, etc.)\n\n\
        Example:\n\
        arky audience create premium-members --data '{\n\
          \"blocks\": [\n\
            {\"key\": \"title\", \"type\": \"localized_text\", \"value\": {\"en\": \"Premium Members\"}},\n\
            {\"key\": \"description\", \"type\": \"markdown\", \"value\": {\"en\": \"Exclusive access\"}}\n\
          ]\n\
        }'")]
    Create {
        /// Audience key (unique within business)
        key: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update an audience
    Update {
        /// Audience ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Delete an audience
    Delete {
        /// Audience ID
        id: String,
    },
    /// List subscribers of an audience
    #[command(long_about = "List accounts subscribed to an audience.\n\n\
        Example:\n\
        arky audience subscribers AUDIENCE_ID --limit 50\n\n\
        Response shape:\n\
        {\"data\": [{\"accountId\": \"...\", \"email\": \"...\", \"subscribedAt\": \"...\"}],\n\
         \"cursor\": \"...\"}")]
    Subscribers {
        /// Audience ID
        id: String,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
    },
}

pub async fn handle(cmd: AudienceCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        AudienceCommand::Get { id } => {
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/audiences/{id}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        AudienceCommand::List {
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
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/audiences"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        AudienceCommand::Create { key, data } => {
            let mut body = json!({ "key": key });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/audiences"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        AudienceCommand::Update { id, data } => {
            let mut body = json!({ "id": id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/audiences/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        AudienceCommand::Delete { id } => {
            let result = client
                .delete(&format!("/v1/businesses/{biz_id}/audiences/{id}"))
                .await?;
            crate::output::print_output(&result, format);
            crate::output::print_success("Audience deleted");
        }
        AudienceCommand::Subscribers { id, limit, cursor } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(
                    &format!("/v1/businesses/{biz_id}/audiences/{id}/subscribers"),
                    &params_ref,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
