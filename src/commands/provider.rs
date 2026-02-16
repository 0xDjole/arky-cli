use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum ProviderCommand {
    /// Get a provider by ID or slug
    #[command(long_about = "Fetch a single service provider.\n\n\
        Example:\n\
        arky provider get PROVIDER_ID\n\n\
        Response shape:\n\
        {\"id\": \"...\", \"key\": \"john-doe\", \"status\": \"active\",\n\
         \"blocks\": [{\"key\": \"name\", \"type\": \"localized_text\", \"value\": {\"en\": \"John Doe\"}}],\n\
         \"concurrentLimit\": 1}")]
    Get {
        /// Provider ID or slug
        id: String,
    },
    /// List providers
    #[command(long_about = "List service providers.\n\n\
        Optionally filter by service to see which providers offer a specific service.\n\n\
        Examples:\n\
        arky provider list\n\
        arky provider list --service-id SVC_ID\n\
        arky provider list --statuses active")]
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, help = "Filter providers by service ID")]
        service_id: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long, help = "Comma-separated: draft,active,archived")]
        statuses: Option<String>,
    },
    /// Create a provider (person/resource that delivers services)
    #[command(long_about = "Create a service provider.\n\n\
        Required:\n\
          KEY (positional)  Provider key — letters, numbers, _ and - only, max 255 chars.\n\n\
        Optional (--data JSON):\n\
          blocks           Profile info blocks (name, bio, avatar) — same types as nodes\n\
          concurrentLimit  How many bookings at once (default: 1 = one at a time)\n\
          status           \"draft\" (default) | \"active\" | \"archived\"\n\n\
        Providers are people or resources that deliver services (e.g., a barber,\n\
        a meeting room, a vehicle).\n\n\
        After creating a provider, link them to services via `arky service create`\n\
        or `arky service update` by adding them to the providers array.\n\n\
        Example:\n\
        arky provider create john-doe --data '{\n\
          \"blocks\": [\n\
            {\"key\": \"name\", \"type\": \"localized_text\", \"value\": {\"en\": \"John Doe\"}},\n\
            {\"key\": \"bio\", \"type\": \"markdown\", \"value\": {\"en\": \"Expert barber with 10 years experience\"}},\n\
            {\"key\": \"avatar\", \"type\": \"relationship_media\", \"value\": {\"id\": \"media_456\"}}\n\
          ],\n\
          \"concurrentLimit\": 1\n\
        }'")]
    Create {
        /// Provider key (unique within business, URL-safe)
        key: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update a provider
    #[command(long_about = "Update a provider by ID.\n\n\
        Optional (--data JSON):\n\
          blocks           Array of blocks — REPLACES entire array\n\
          concurrentLimit  Max simultaneous bookings\n\
          status           \"draft\" | \"active\" | \"archived\"\n\n\
        Example:\n\
        arky provider update PROV_ID --data '{\"blocks\": [...], \"concurrentLimit\": 2}'\n\
        arky provider update PROV_ID --data '{\"status\": \"active\"}'")]
    Update {
        /// Provider ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Delete a provider
    Delete {
        /// Provider ID
        id: String,
    },
    /// Get a provider's working time for a service
    #[command(long_about = "Fetch the working time schedule for a provider.\n\n\
        Optionally filter by service ID to see the schedule for a specific service.\n\n\
        Example:\n\
        arky provider working-time PROVIDER_ID\n\
        arky provider working-time PROVIDER_ID --service-id SVC_ID\n\n\
        Response shape:\n\
        {\"workingDays\": [{\"day\": \"monday\", \"workingHours\": [{\"from\": 32400000, \"to\": 61200000}]}],\n\
         \"outcastDates\": [], \"specificDates\": []}")]
    WorkingTime {
        /// Provider ID
        provider_id: String,
        #[arg(long)]
        service_id: Option<String>,
    },
}

pub async fn handle(cmd: ProviderCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        ProviderCommand::Get { id } => {
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/providers/{id}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        ProviderCommand::List {
            query,
            service_id,
            limit,
            cursor,
            statuses,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref q) = query {
                params.push(("query", q.clone()));
            }
            if let Some(ref s) = service_id {
                params.push(("serviceId", s.clone()));
            }
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            if let Some(ref s) = statuses {
                params.push(("statuses", s.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/providers"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        ProviderCommand::Create { key, data } => {
            let mut body = json!({ "key": key });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/providers"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        ProviderCommand::Update { id, data } => {
            let mut body = json!({ "id": id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/providers/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        ProviderCommand::Delete { id } => {
            let result = client
                .delete(&format!("/v1/businesses/{biz_id}/providers/{id}"))
                .await?;
            crate::output::print_output(&result, format);
            crate::output::print_success("Provider deleted");
        }
        ProviderCommand::WorkingTime {
            provider_id,
            service_id,
        } => {
            let mut params: Vec<(&str, String)> = vec![];
            if let Some(ref s) = service_id {
                params.push(("serviceId", s.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(
                    &format!("/v1/businesses/{biz_id}/providers/{provider_id}/working-time"),
                    &params_ref,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
