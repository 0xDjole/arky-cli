use crate::client::ArkyClient;
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum NotificationCommand {
    /// Get email delivery statistics
    #[command(name = "delivery-stats", long_about = "Get email delivery statistics for the business.\n\n\
        Shows sent, delivered, opened, bounced counts.\n\n\
        Example:\n\
        arky notification delivery-stats")]
    DeliveryStats,
}

pub async fn handle(
    cmd: NotificationCommand,
    client: &ArkyClient,
    format: &Format,
) -> Result<()> {
    match cmd {
        NotificationCommand::DeliveryStats => {
            let biz_id = client.require_business_id()?;
            let result = client
                .get(
                    &format!("/v1/notifications/track/stats/{biz_id}"),
                    &[],
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
