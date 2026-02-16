use crate::client::ArkyClient;
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum PlatformCommand {
    /// List supported currencies
    #[command(long_about = "List all currencies supported by the platform.\n\n\
        Example:\n\
        arky platform currencies")]
    Currencies,
    /// List available integration services
    #[command(long_about = "List all integration services available on the platform.\n\n\
        Example:\n\
        arky platform integrations")]
    Integrations,
    /// List countries and their states/regions
    #[command(long_about = "List all countries with their states/regions.\n\n\
        Example:\n\
        arky platform countries\n\n\
        Response shape:\n\
        {\"items\": [{\"code\": \"US\", \"name\": \"United States\", \"states\": [{\"code\": \"NY\", \"name\": \"New York\"}]}]}")]
    Countries,
    /// Get a specific country with states
    #[command(long_about = "Fetch a specific country by ISO code.\n\n\
        Example:\n\
        arky platform country US\n\n\
        Response shape:\n\
        {\"code\": \"US\", \"name\": \"United States\", \"states\": [...]}")]
    Country {
        /// ISO country code (e.g. US, GB, DE)
        code: String,
    },
    /// List available webhook event types
    #[command(name = "webhook-events", long_about = "List all event types available for webhooks.\n\n\
        Example:\n\
        arky platform webhook-events\n\n\
        Response: array of event type strings (e.g. \"order.paid\", \"booking.confirmed\")")]
    WebhookEvents,
}

pub async fn handle(cmd: PlatformCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    match cmd {
        PlatformCommand::Currencies => {
            let result = client.get("/v1/platform/currencies", &[]).await?;
            crate::output::print_output(&result, format);
        }
        PlatformCommand::Integrations => {
            let result = client
                .get("/v1/platform/integration-services", &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        PlatformCommand::Countries => {
            let result = client.get("/v1/platform/countries", &[]).await?;
            crate::output::print_output(&result, format);
        }
        PlatformCommand::Country { code } => {
            let result = client
                .get(&format!("/v1/platform/countries/{code}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        PlatformCommand::WebhookEvents => {
            let result = client.get("/v1/platform/events", &[]).await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
