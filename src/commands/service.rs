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
    Required:\n\
      KEY (positional)   Service key — letters, numbers, _ and - only, max 255 chars.\n\n\
    Required (--data JSON):\n\
      slug            Localized slug: {\"en\": \"haircut\"}\n\
      status          \"draft\" | \"active\" | \"archived\"\n\
      networkIds      Array of network IDs (use [] if none)\n\
      audienceIds     Array of audience IDs (use [] if none)\n\
      filters         Array of filter objects (use [] if none)\n\
      blocks          Array of content blocks (each needs type, id, key, properties, value)\n\
      providers       At least one provider object (see below)\n\n\
    Provider fields (each object in providers array):\n\
      providerId          ID of an existing provider (create one first with `arky provider create`)\n\
      prices              [{\"currency\": \"usd\", \"market\": \"us\", \"amount\": 5000}] (amount in cents)\n\
      durations           [{\"duration\": 60, \"isPause\": false}] (minutes, 60 = 1 hour)\n\
      isApprovalRequired  false (whether bookings need manual approval)\n\
      audienceIds         [] (audience restrictions for this provider)\n\
      workingTime         Schedule when this provider offers this service (see below)\n\n\
    Working time structure:\n\
      workingDays: [{\"day\": \"monday\", \"workingHours\": [{\"from\": 540, \"to\": 1020}]}]\n\
        day: monday|tuesday|wednesday|thursday|friday|saturday|sunday\n\
        from/to: minutes from midnight (540 = 9:00 AM, 1020 = 5:00 PM)\n\
      outcastDates: [] — holidays/blocked dates\n\
      specificDates: [] — overrides\n\n\
    Common time values (minutes from midnight):\n\
      6:00 AM = 360     9:00 AM = 540      12:00 PM = 720\n\
      1:00 PM = 780     5:00 PM = 1020      9:00 PM = 1260\n\n\
    Common duration values (minutes):\n\
      15 min = 15    30 min = 30    45 min = 45    60 min = 60\n\n\
    Working example (from integration tests):\n\
    arky service create haircut --data '{\n\
      \"slug\": {\"en\": \"haircut\"},\n\
      \"status\": \"active\",\n\
      \"networkIds\": [],\n\
      \"audienceIds\": [],\n\
      \"filters\": [],\n\
      \"blocks\": [\n\
        {\"type\": \"localized_text\", \"id\": \"b1\", \"key\": \"title\", \"properties\": {}, \"value\": {\"en\": \"Test Service\"}}\n\
      ],\n\
      \"providers\": [{\n\
        \"providerId\": \"PROVIDER_ID\",\n\
        \"prices\": [{\"currency\": \"usd\", \"market\": \"us\", \"amount\": 5000}],\n\
        \"durations\": [{\"duration\": 60, \"isPause\": false}],\n\
        \"isApprovalRequired\": false,\n\
        \"audienceIds\": [],\n\
        \"workingTime\": {\n\
          \"workingDays\": [\n\
            {\"day\": \"monday\", \"workingHours\": [{\"from\": 540, \"to\": 1020}]},\n\
            {\"day\": \"tuesday\", \"workingHours\": [{\"from\": 540, \"to\": 1020}]},\n\
            {\"day\": \"wednesday\", \"workingHours\": [{\"from\": 540, \"to\": 1020}]},\n\
            {\"day\": \"thursday\", \"workingHours\": [{\"from\": 540, \"to\": 1020}]},\n\
            {\"day\": \"friday\", \"workingHours\": [{\"from\": 540, \"to\": 1020}]}\n\
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
        Optional (--data JSON):\n\
          blocks     Array of blocks — REPLACES entire array, include all you want to keep\n\
          providers  Array of providers — REPLACES entire array (at least 1 required)\n\
          filters    Array of filters — REPLACES entire array\n\
          status     \"draft\" | \"active\" | \"archived\"\n\n\
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
            let _ = client
                .delete(&format!("/v1/businesses/{biz_id}/services/{id}"))
                .await?;
            crate::output::print_success("Service deleted");
        }
    }
    Ok(())
}
