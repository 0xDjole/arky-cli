use crate::client::ArkyClient;
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum NetworkCommand {
    /// Search services across a network
    #[command(name = "search-services", long_about = "Search for services across a network.\n\n\
        Required:\n\
          NETWORK_KEY (positional)  The network key to search within.\n\n\
        Optional:\n\
          --query, --limit, --cursor, --statuses, --sort-field, --sort-direction\n\n\
        Example:\n\
        arky network search-services my-network --query \"haircut\" --limit 10")]
    SearchServices {
        /// Network key
        network_key: String,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        statuses: Option<String>,
        #[arg(long)]
        sort_field: Option<String>,
        #[arg(long)]
        sort_direction: Option<String>,
    },
    /// Search products across a network
    #[command(name = "search-products", long_about = "Search for products across a network.\n\n\
        Required:\n\
          NETWORK_KEY (positional)  The network key to search within.\n\n\
        Optional:\n\
          --query, --limit, --cursor, --statuses, --sort-field, --sort-direction,\n\
          --price-from, --price-to (cents)\n\n\
        Example:\n\
        arky network search-products my-network --query \"shirt\" --price-from 1000 --price-to 5000")]
    SearchProducts {
        /// Network key
        network_key: String,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        statuses: Option<String>,
        #[arg(long)]
        sort_field: Option<String>,
        #[arg(long)]
        sort_direction: Option<String>,
        #[arg(long, help = "Minimum price in cents")]
        price_from: Option<u64>,
        #[arg(long, help = "Maximum price in cents")]
        price_to: Option<u64>,
    },
    /// Search providers across a network
    #[command(name = "search-providers", long_about = "Search for providers across a network.\n\n\
        Required:\n\
          NETWORK_KEY (positional)  The network key to search within.\n\n\
        Optional:\n\
          --query, --limit, --cursor, --statuses, --sort-field, --sort-direction\n\n\
        Example:\n\
        arky network search-providers my-network --query \"john\"")]
    SearchProviders {
        /// Network key
        network_key: String,
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        statuses: Option<String>,
        #[arg(long)]
        sort_field: Option<String>,
        #[arg(long)]
        sort_direction: Option<String>,
    },
}

pub async fn handle(cmd: NetworkCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    match cmd {
        NetworkCommand::SearchServices {
            network_key,
            query,
            limit,
            cursor,
            statuses,
            sort_field,
            sort_direction,
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
            if let Some(ref sf) = sort_field {
                params.push(("sortField", sf.clone()));
            }
            if let Some(ref sd) = sort_direction {
                params.push(("sortDirection", sd.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(
                    &format!("/v1/networks/{network_key}/services"),
                    &params_ref,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        NetworkCommand::SearchProducts {
            network_key,
            query,
            limit,
            cursor,
            statuses,
            sort_field,
            sort_direction,
            price_from,
            price_to,
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
            if let Some(ref sf) = sort_field {
                params.push(("sortField", sf.clone()));
            }
            if let Some(ref sd) = sort_direction {
                params.push(("sortDirection", sd.clone()));
            }
            if let Some(pf) = price_from {
                params.push(("priceFrom", pf.to_string()));
            }
            if let Some(pt) = price_to {
                params.push(("priceTo", pt.to_string()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(
                    &format!("/v1/networks/{network_key}/products"),
                    &params_ref,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        NetworkCommand::SearchProviders {
            network_key,
            query,
            limit,
            cursor,
            statuses,
            sort_field,
            sort_direction,
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
            if let Some(ref sf) = sort_field {
                params.push(("sortField", sf.clone()));
            }
            if let Some(ref sd) = sort_direction {
                params.push(("sortDirection", sd.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(
                    &format!("/v1/networks/{network_key}/providers"),
                    &params_ref,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
