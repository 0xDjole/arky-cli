use crate::client::ArkyClient;
use crate::commands::parse_data;
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum NotificationCommand {
    /// Trigger a notification (send email to recipients or audience)
    #[command(long_about = "Send a notification via the trigger API.\n\n\
        Required (--data JSON):\n\
          channel      Only \"email\" currently supported (required)\n\
          businessId   Business ID (required, auto-set from config)\n\
          nodeId       CMS node ID containing the email template (required for email)\n\n\
        Optional:\n\
          recipients   Array of email addresses: [\"user@example.com\"]\n\
          audienceId   Single audience ID — resolves all subscribers as recipients\n\
          fromName     Sender display name (defaults to \"Arky\")\n\
          vars         Template variables object: {\"subject\": \"Hello\", \"name\": \"World\"}\n\n\
        You must provide either \"recipients\" or \"audienceId\" (or both).\n\n\
        Example — send to specific emails:\n\
        arky notification trigger --data '{\n\
          \"channel\": \"email\",\n\
          \"recipients\": [\"user@example.com\"],\n\
          \"nodeId\": \"NODE_ID_WITH_EMAIL_TEMPLATE\",\n\
          \"fromName\": \"My App\",\n\
          \"vars\": {\"subject\": \"Welcome!\", \"name\": \"User\"}\n\
        }'\n\n\
        Example — send to all audience subscribers:\n\
        arky notification trigger --data '{\n\
          \"channel\": \"email\",\n\
          \"audienceId\": \"AUDIENCE_ID\",\n\
          \"nodeId\": \"NEWSLETTER_TEMPLATE_NODE_ID\",\n\
          \"fromName\": \"Newsletter\",\n\
          \"vars\": {\"subject\": \"Weekly Update\", \"content\": \"Here is the news...\"}\n\
        }'")]
    Trigger {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
}

pub async fn handle(
    cmd: NotificationCommand,
    client: &ArkyClient,
    format: &Format,
) -> Result<()> {
    match cmd {
        NotificationCommand::Trigger { data } => {
            let biz_id = client.require_business_id()?;
            let mut body = parse_data(data.as_deref())?;
            if body.get("businessId").is_none() {
                body["businessId"] = serde_json::json!(biz_id);
            }
            let result = client
                .post("/v1/notifications/trigger", &body)
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
