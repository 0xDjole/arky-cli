use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum ServiceCommand {
    /// Get a service by ID or slug
    #[command(long_about = "Fetch a single bookable service.\n\n\
        Example:\n\
        arky service get SERVICE_ID\n\n\
        Response shape:\n\
        {\"id\": \"...\", \"key\": \"haircut\", \"status\": \"active\",\n\
         \"blocks\": [{\"key\": \"title\", \"type\": \"localized_text\", \"value\": {\"en\": \"Haircut\"}}],\n\
         \"providers\": [{\"providerId\": \"prov_123\", \"prices\": [...], \"durations\": [...],\n\
           \"workingTime\": {...}}]}")]
    Get {
        /// Service ID or slug
        id: String,
    },
    /// List services
    #[command(long_about = "List bookable services.\n\n\
        Examples:\n\
        arky service list\n\
        arky service list --query \"hair\" --statuses active\n\n\
        Response: {\"data\": [...], \"cursor\": \"...\"}")]
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long, help = "Comma-separated: draft,active,archived")]
        statuses: Option<String>,
    },
    /// Create a service with blocks, providers, and working time
    #[command(long_about = "Create a bookable service.\n\n\
        Services define what can be booked. They have:\n\
          - blocks: content (title, description, images) — same as nodes\n\
          - providers: who delivers the service, with prices, durations, and schedules\n\n\
        Provider fields:\n\
          providerId: ID of an existing provider (create one first with `arky provider create`)\n\
          prices: [{\"currency\": \"USD\", \"market\": \"us\", \"amount\": 2500}] (amount in cents)\n\
          durations: [{\"duration\": 1800000}] (in milliseconds, 1800000 = 30 min)\n\
          workingTime: schedule when this provider offers this service\n\n\
        Working time structure:\n\
          workingDays: [{\"day\": \"monday\", \"workingHours\": [{\"from\": 32400000, \"to\": 61200000}]}]\n\
            day: monday|tuesday|wednesday|thursday|friday|saturday|sunday\n\
            from/to: milliseconds from midnight (32400000 = 9:00 AM, 61200000 = 5:00 PM)\n\
          outcastDates: [{\"from\": timestamp, \"to\": timestamp}] — holidays/blocked dates\n\
          specificDates: [{\"date\": timestamp, \"workingHours\": [...]}] — overrides\n\n\
        Common time values (ms from midnight):\n\
          6:00 AM = 21600000    9:00 AM = 32400000    12:00 PM = 43200000\n\
          1:00 PM = 46800000    5:00 PM = 61200000     9:00 PM = 75600000\n\n\
        Common duration values:\n\
          15 min = 900000    30 min = 1800000    45 min = 2700000    60 min = 3600000\n\n\
        Example:\n\
        arky service create haircut --data '{\n\
          \"blocks\": [\n\
            {\"key\": \"title\", \"type\": \"localized_text\", \"value\": {\"en\": \"Haircut\"}},\n\
            {\"key\": \"description\", \"type\": \"markdown\", \"value\": {\"en\": \"Professional haircut\"}}\n\
          ],\n\
          \"providers\": [{\n\
            \"providerId\": \"PROVIDER_ID\",\n\
            \"prices\": [{\"currency\": \"USD\", \"market\": \"us\", \"amount\": 2500}],\n\
            \"durations\": [{\"duration\": 1800000}],\n\
            \"workingTime\": {\n\
              \"workingDays\": [\n\
                {\"day\": \"monday\", \"workingHours\": [{\"from\": 32400000, \"to\": 61200000}]},\n\
                {\"day\": \"tuesday\", \"workingHours\": [{\"from\": 32400000, \"to\": 61200000}]},\n\
                {\"day\": \"wednesday\", \"workingHours\": [{\"from\": 32400000, \"to\": 61200000}]},\n\
                {\"day\": \"thursday\", \"workingHours\": [{\"from\": 32400000, \"to\": 61200000}]},\n\
                {\"day\": \"friday\", \"workingHours\": [{\"from\": 32400000, \"to\": 61200000}]}\n\
              ],\n\
              \"outcastDates\": [],\n\
              \"specificDates\": []\n\
            }\n\
          }]\n\
        }'")]
    Create {
        /// Service key (unique within business, URL-safe)
        key: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update a service
    #[command(long_about = "Update a service by ID.\n\n\
        Blocks and providers replace entirely. Include all you want to keep.\n\n\
        Example:\n\
        arky service update SVC_ID --data '{\"blocks\": [...], \"providers\": [...]}'\n\
        arky service update SVC_ID --data '{\"status\": \"active\"}'")]
    Update {
        /// Service ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Delete a service
    Delete {
        /// Service ID
        id: String,
    },
}

pub async fn handle(cmd: ServiceCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        ServiceCommand::Get { id } => {
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/services/{id}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        ServiceCommand::List {
            query,
            limit,
            cursor,
            statuses,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref q) = query {
                params.push(("query", q.clone()));
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
                .get(&format!("/v1/businesses/{biz_id}/services"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        ServiceCommand::Create { key, data } => {
            let mut body = json!({ "key": key });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/services"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        ServiceCommand::Update { id, data } => {
            let mut body = json!({ "id": id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/services/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        ServiceCommand::Delete { id } => {
            let result = client
                .delete(&format!("/v1/businesses/{biz_id}/services/{id}"))
                .await?;
            crate::output::print_output(&result, format);
            crate::output::print_success("Service deleted");
        }
    }
    Ok(())
}
