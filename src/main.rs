mod client;
mod commands;
mod config;
mod error;
mod output;

use clap::{Parser, Subcommand};
use commands::{
    account::AccountCommand, audience::AudienceCommand, auth::AuthCommand,
    booking::BookingCommand, business::BusinessCommand, config_cmd::ConfigCommand,
    database::DatabaseCommand, media::MediaCommand,
    network::NetworkCommand, node::NodeCommand, notification::NotificationCommand,
    order::OrderCommand, platform::PlatformCommand, product::ProductCommand,
    promo_code::PromoCodeCommand, provider::ProviderCommand, service::ServiceCommand,
    shipping::ShippingCommand, workflow::WorkflowCommand,
};

/// Arky CLI — agentic-first CLI for the Arky platform.
///
/// IMPORTANT: Always run `arky <command> <action> --help` before using any command.
/// Each action's --help documents ALL required/optional fields with working JSON examples
/// that are tested against the live API. Read --help first, get it right first try.
///
/// Quick reference:
///   arky node create --help          # see exact JSON shape for creating nodes
///   arky product create --help       # see required fields for products
///   arky notification trigger --help  # see how to send emails
///
/// Setup:
///   arky config set base_url http://localhost:8000
///   arky config set business_id YOUR_BUSINESS_ID
///   arky auth login your@email.com          # sends verification code
///   arky auth verify your@email.com CODE    # saves token automatically
///
/// Or via environment variables:
///   export ARKY_BASE_URL=http://localhost:8000
///   export ARKY_BUSINESS_ID=your-business-id
///   export ARKY_TOKEN=your-api-token
///
/// Authentication:
///   Method 1: Email magic link (arky auth login + arky auth verify)
///   Method 2: API token via --token flag or ARKY_TOKEN env var
///   Method 3: Anonymous session (arky auth session)
///
/// Data input (--data flag):
///   Inline JSON:  --data '{"key": "value"}'
///   From file:    --data @content.json
///   From stdin:   echo '{}' | arky <cmd> --data -
///
/// Output formats (--format):
///   json   - Pretty JSON (default, best for AI agents)
///   table  - Human-readable table
///   plain  - Key=value pairs for piping
///
/// Block system (used by nodes, products, services, providers):
///   Every block needs: type, id, key, properties (usually {}), value
///   Types: localized_text, markdown, number, boolean, text, list, map,
///          relationship_entry, relationship_media, geo_location
#[derive(Parser, Debug)]
#[command(name = "arky", version, about, long_about)]
struct Cli {
    /// Server base URL
    #[arg(long, global = true, env = "ARKY_BASE_URL")]
    base_url: Option<String>,

    /// Business ID
    #[arg(long, global = true, env = "ARKY_BUSINESS_ID")]
    business_id: Option<String>,

    /// Auth token
    #[arg(long, global = true, env = "ARKY_TOKEN")]
    token: Option<String>,

    /// Output format: json (default), table, plain
    #[arg(long, global = true, env = "ARKY_FORMAT", default_value = "json")]
    format: Option<String>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Authentication: login, verify, session
    Auth {
        #[command(subcommand)]
        cmd: AuthCommand,
    },
    /// Manage CLI configuration
    Config {
        #[command(subcommand)]
        cmd: ConfigCommand,
    },
    /// Manage businesses
    Business {
        #[command(subcommand)]
        cmd: BusinessCommand,
    },
    /// Manage content nodes (CMS: pages, blog posts, newsletters)
    Node {
        #[command(subcommand)]
        cmd: NodeCommand,
    },
    /// Manage products (e-shop)
    Product {
        #[command(subcommand)]
        cmd: ProductCommand,
    },
    /// Manage orders (e-shop)
    Order {
        #[command(subcommand)]
        cmd: OrderCommand,
    },
    /// Manage workflows (DAG-based automation)
    Workflow {
        #[command(subcommand)]
        cmd: WorkflowCommand,
    },
    /// Manage bookable services
    Service {
        #[command(subcommand)]
        cmd: ServiceCommand,
    },
    /// Manage service providers (people/resources)
    Provider {
        #[command(subcommand)]
        cmd: ProviderCommand,
    },
    /// Manage bookings
    Booking {
        #[command(subcommand)]
        cmd: BookingCommand,
    },
    /// Key-value database operations
    Db {
        #[command(subcommand)]
        cmd: DatabaseCommand,
    },
    /// Manage media files
    Media {
        #[command(subcommand)]
        cmd: MediaCommand,
    },
    /// Manage audiences (access groups & subscriptions)
    Audience {
        #[command(subcommand)]
        cmd: AudienceCommand,
    },
    /// Manage promo/discount codes
    #[command(name = "promo-code")]
    PromoCode {
        #[command(subcommand)]
        cmd: PromoCodeCommand,
    },
    /// Shipping: rates and label purchase
    Shipping {
        #[command(subcommand)]
        cmd: ShippingCommand,
    },
    /// Manage your account
    Account {
        #[command(subcommand)]
        cmd: AccountCommand,
    },
    /// Platform info: currencies, countries, integrations
    Platform {
        #[command(subcommand)]
        cmd: PlatformCommand,
    },
    /// Search across networks
    Network {
        #[command(subcommand)]
        cmd: NetworkCommand,
    },
    /// Notification & email tracking
    Notification {
        #[command(subcommand)]
        cmd: NotificationCommand,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let resolved = config::Config::resolve(
        cli.base_url.as_deref(),
        cli.business_id.as_deref(),
        cli.token.as_deref(),
        cli.format.as_deref(),
    );

    let format = output::Format::from_str(&resolved.format);

    let client = client::ArkyClient::new(
        resolved.base_url.clone(),
        resolved.business_id.clone(),
        resolved.token.clone(),
    );

    let result = match cli.command {
        Command::Auth { cmd } => commands::auth::handle(cmd, &client, &format).await,
        Command::Config { cmd } => commands::config_cmd::handle(cmd, &resolved, &format).await,
        Command::Business { cmd } => commands::business::handle(cmd, &client, &format).await,
        Command::Node { cmd } => commands::node::handle(cmd, &client, &format).await,
        Command::Product { cmd } => commands::product::handle(cmd, &client, &format).await,
        Command::Order { cmd } => commands::order::handle(cmd, &client, &format).await,
        Command::Workflow { cmd } => commands::workflow::handle(cmd, &client, &format).await,
        Command::Service { cmd } => commands::service::handle(cmd, &client, &format).await,
        Command::Provider { cmd } => commands::provider::handle(cmd, &client, &format).await,
        Command::Booking { cmd } => commands::booking::handle(cmd, &client, &format).await,
        Command::Db { cmd } => commands::database::handle(cmd, &client, &format).await,
        Command::Media { cmd } => commands::media::handle(cmd, &client, &format).await,
        Command::Audience { cmd } => commands::audience::handle(cmd, &client, &format).await,
        Command::PromoCode { cmd } => commands::promo_code::handle(cmd, &client, &format).await,
        Command::Shipping { cmd } => commands::shipping::handle(cmd, &client, &format).await,
        Command::Account { cmd } => commands::account::handle(cmd, &client, &format).await,
        Command::Platform { cmd } => commands::platform::handle(cmd, &client, &format).await,
        Command::Network { cmd } => commands::network::handle(cmd, &client, &format).await,
        Command::Notification { cmd } => {
            commands::notification::handle(cmd, &client, &format).await
        }
    };

    if let Err(e) = result {
        output::print_error(&e.to_string());
        std::process::exit(1);
    }
}
