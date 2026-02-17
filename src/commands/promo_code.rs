use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum PromoCodeCommand {
    /// Get a promo code by ID
    #[command(long_about = "Fetch a single promo code.\n\n\
        Example:\n\
        arky promo-code get PROMO_ID\n\n\
        Response shape:\n\
        {\"id\": \"...\", \"code\": \"SUMMER20\", \"status\": \"active\",\n\
         \"discounts\": [{\"type\": \"items_percentage\", \"bps\": 2000, \"marketId\": \"us\"}],\n\
         \"conditions\": [{\"type\": \"max_uses\", \"value\": 100}],\n\
         \"usageCount\": 15}")]
    Get {
        /// Promo code ID
        id: String,
    },
    /// List promo codes
    #[command(long_about = "List promo codes with optional filters.\n\n\
        Examples:\n\
        arky promo-code list\n\
        arky promo-code list --query \"SUMMER\" --statuses active")]
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long, help = "Comma-separated: active,expired,disabled")]
        statuses: Option<String>,
    },
    /// Create a promo code
    #[command(long_about = "Create a discount promo code.\n\n\
        Required (--data JSON):\n\
          code        Promo code string (e.g. \"SUMMER20\")\n\
          discounts   At least one discount object (see types below)\n\n\
        Optional:\n\
          conditions  Array of restriction rules (see types below)\n\n\
        Discount types:\n\
          items_percentage      Percentage off items. bps = basis points (1000 = 10%, 2000 = 20%)\n\
          items_fixed           Fixed amount off items (in cents)\n\
          shipping_percentage   Percentage off shipping costs\n\n\
        Discount object fields:\n\
          type      One of the discount types above (required)\n\
          marketId  Market this discount applies to (required)\n\
          bps       Basis points for percentage types (required for percentage types)\n\
          amount    Fixed amount in cents (required for items_fixed)\n\n\
        Condition types:\n\
          products          Array of product IDs this code applies to\n\
          services          Array of service IDs this code applies to\n\
          min_order_amount  Minimum order amount (in cents)\n\
          date_range        {\"start\": timestamp, \"end\": timestamp}\n\
          max_uses          Maximum total uses across all customers\n\
          max_uses_per_user Maximum uses per individual customer\n\n\
        Condition value format (adjacently tagged):\n\
          max_uses:          {\"type\": \"count\", \"value\": 50}\n\
          max_uses_per_user: {\"type\": \"count\", \"value\": 5}\n\
          min_order_amount:  {\"type\": \"amount\", \"value\": 5000}\n\
          products:          {\"type\": \"ids\", \"value\": [\"prod_1\", \"prod_2\"]}\n\
          date_range:        {\"type\": \"range\", \"value\": {\"start\": 1704067200000, \"end\": 1706745600000}}\n\n\
        Working example (from integration tests):\n\
        arky promo-code create --data '{\n\
          \"code\": \"SUMMER20\",\n\
          \"discounts\": [{\"type\": \"items_percentage\", \"marketId\": \"us\", \"bps\": 1500}],\n\
          \"conditions\": [{\"type\": \"max_uses\", \"value\": {\"type\": \"count\", \"value\": 50}}]\n\
        }'")]
    Create {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update a promo code
    #[command(long_about = "Update a promo code by ID.\n\n\
        Example:\n\
        arky promo-code update PROMO_ID --data '{\"conditions\": [{\"type\": \"max_uses\", \"value\": 200}]}'")]
    Update {
        /// Promo code ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Delete a promo code
    Delete {
        /// Promo code ID
        id: String,
    },
}

pub async fn handle(cmd: PromoCodeCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        PromoCodeCommand::Get { id } => {
            let result = client
                .get(
                    &format!("/v1/businesses/{biz_id}/promo-codes/{id}"),
                    &[],
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        PromoCodeCommand::List {
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
                .get(
                    &format!("/v1/businesses/{biz_id}/promo-codes"),
                    &params_ref,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        PromoCodeCommand::Create { data } => {
            let mut body = json!({ "businessId": biz_id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .post(
                    &format!("/v1/businesses/{biz_id}/promo-codes"),
                    &body,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        PromoCodeCommand::Update { id, data } => {
            let mut body = json!({ "id": id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .put(
                    &format!("/v1/businesses/{biz_id}/promo-codes/{id}"),
                    &body,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        PromoCodeCommand::Delete { id } => {
            let result = client
                .delete(&format!("/v1/businesses/{biz_id}/promo-codes/{id}"))
                .await?;
            crate::output::print_output(&result, format);
            crate::output::print_success("Promo code deleted");
        }
    }
    Ok(())
}
