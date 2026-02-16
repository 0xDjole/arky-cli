use crate::client::ArkyClient;
use crate::commands::parse_data;
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum EventCommand {
    /// List events for an entity
    #[command(long_about = "List event history for a specific entity.\n\n\
        Events track changes to orders, bookings, and other entities.\n\
        Pass the entity ID (e.g., order ID, booking ID) to see its history.\n\n\
        Examples:\n\
        arky event list ORDER_ID\n\
        arky event list BOOKING_ID --limit 50\n\n\
        Response shape:\n\
        {\"data\": [{\"id\": \"...\", \"type\": \"order.paid\", \"entity\": \"order_123\",\n\
          \"data\": {...}, \"createdAt\": \"...\"}], \"cursor\": \"...\"}")]
    List {
        /// Entity identifier (e.g., order ID, booking ID)
        entity: String,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Update an event
    #[command(long_about = "Update an event by ID.\n\n\
        Required (--data JSON):\n\
          event   Event action object with \"action\" field\n\n\
        Event action examples:\n\
          {\"action\": \"order_status_changed\", \"data\": {\"from\": \"pending\", \"to\": \"paid\"}}\n\
          {\"action\": \"booking_confirmed\"}\n\n\
        Example:\n\
        arky event update EVT_ID --data '{\"event\": {\"action\": \"order_updated\"}}'")]
    Update {
        /// Event ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
}

pub async fn handle(cmd: EventCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        EventCommand::List {
            entity,
            limit,
            cursor,
        } => {
            let mut params: Vec<(&str, String)> = vec![
                ("entity", entity),
                ("limit", limit.to_string()),
            ];
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/events"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        EventCommand::Update { id, data } => {
            let body = parse_data(data.as_deref())?;
            let result = client
                .put(&format!("/v1/events/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
