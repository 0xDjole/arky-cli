use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum OrderCommand {
    /// Get an order by ID
    #[command(long_about = "Fetch a single order with all details.\n\n\
        Example:\n\
        arky order get ORDER_ID\n\n\
        Response includes: id, status, items, totals, addresses, payments, shipments.")]
    Get {
        /// Order ID
        id: String,
    },
    /// List orders
    #[command(long_about = "List orders with optional filters.\n\n\
        Statuses: pending, paid, shipped, delivered, cancelled, refunded.\n\n\
        Examples:\n\
        arky order list\n\
        arky order list --status paid --limit 10\n\
        arky order list --account-id ACC_ID\n\
        arky order list --sort-field createdAt --sort-direction desc")]
    List {
        #[arg(long, help = "Filter: pending, paid, shipped, delivered, cancelled, refunded")]
        status: Option<String>,
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        account_id: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long)]
        sort_field: Option<String>,
        #[arg(long)]
        sort_direction: Option<String>,
    },
    /// Create an order manually
    #[command(long_about = "Create an order manually (admin use).\n\n\
        For normal checkout flow, use `arky order checkout` instead.\n\n\
        Required (--data JSON):\n\
          items    At least one order item (see fields below).\n\
          market   Market identifier (e.g. \"us\", \"eu\")\n\n\
        Optional:\n\
          status           \"pending\" (default) | \"paid\" | \"shipped\" | \"delivered\" | \"cancelled\" | \"refunded\"\n\
          shippingAddress  {\"name\": \"...\", \"street1\": \"...\", \"city\": \"...\", \"country\": \"...\"}\n\
          billingAddress   Same shape, or {\"sameAsShipping\": true}\n\n\
        Item fields:\n\
          productId   Product ID (required)\n\
          variantKey  Variant key, e.g. \"default\", \"small\" (required)\n\
          quantity    Number of units (required)\n\n\
        Example:\n\
        arky order create --data '{\n\
          \"items\": [{\"productId\": \"prod_123\", \"variantKey\": \"default\", \"quantity\": 1}],\n\
          \"market\": \"us\",\n\
          \"status\": \"paid\"\n\
        }'")]
    Create {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update an order
    #[command(long_about = "Update an order (e.g., change status, add notes).\n\n\
        Optional (--data JSON):\n\
          status   \"pending\" | \"paid\" | \"shipped\" | \"delivered\" | \"cancelled\" | \"refunded\"\n\n\
        Example:\n\
        arky order update ORDER_ID --data '{\"status\": \"shipped\"}'")]
    Update {
        /// Order ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Get a price quote for items
    #[command(long_about = "Calculate prices for a set of items without creating an order.\n\n\
        Use this to preview totals, taxes, and discounts before checkout.\n\n\
        Required (--data JSON):\n\
          items    At least one item (productId, variantKey, quantity).\n\
          market   Market identifier (e.g. \"us\")\n\n\
        Optional:\n\
          promoCode         Promo code string\n\
          shippingAddress   For shipping cost calculation\n\
          shippingMethodId  Shipping method ID\n\n\
        Example:\n\
        arky order quote --data '{\n\
          \"items\": [{\"productId\": \"prod_123\", \"variantKey\": \"default\", \"quantity\": 2}],\n\
          \"promoCode\": \"SAVE10\",\n\
          \"market\": \"us\"\n\
        }'\n\n\
        Response shape:\n\
        {\"subtotal\": 5998, \"discount\": 600, \"tax\": 0, \"total\": 5398,\n\
         \"currency\": \"USD\", \"items\": [...]}")]
    Quote {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Checkout: create order and process payment
    #[command(long_about = "Create an order and process payment in one step.\n\n\
        This is the primary purchase flow.\n\n\
        Required (--data JSON):\n\
          items    At least one item (productId, variantKey, quantity).\n\n\
        Optional:\n\
          market            Market identifier (auto-set from business if omitted)\n\
          paymentMethodId   Payment method ID\n\
          shippingAddress   {\"name\": \"...\", \"street1\": \"...\", \"city\": \"...\", \"state\": \"...\",\n\
                            \"postalCode\": \"...\", \"country\": \"US\"}\n\
          billingAddress    Same shape, or {\"sameAsShipping\": true}\n\
          promoCodeId       Promo code ID for discount\n\
          shippingMethodId  Shipping method ID\n\n\
        Item fields:\n\
          productId   Product ID (required)\n\
          variantKey  Variant key (required)\n\
          quantity    Number of units (required)\n\n\
        Example:\n\
        arky order checkout --data '{\n\
          \"items\": [{\"productId\": \"prod_123\", \"variantKey\": \"default\", \"quantity\": 1}],\n\
          \"paymentMethodId\": \"pm_card_visa\",\n\
          \"market\": \"us\",\n\
          \"shippingAddress\": {\n\
            \"name\": \"John Doe\", \"street1\": \"123 Main St\",\n\
            \"city\": \"NYC\", \"state\": \"NY\", \"postalCode\": \"10001\", \"country\": \"US\"\n\
          },\n\
          \"billingAddress\": {\"sameAsShipping\": true}\n\
        }'")]
    Checkout {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
}

pub async fn handle(cmd: OrderCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        OrderCommand::Get { id } => {
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/orders/{id}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        OrderCommand::List {
            status,
            query,
            account_id,
            limit,
            cursor,
            sort_field,
            sort_direction,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref s) = status {
                params.push(("statuses", s.clone()));
            }
            if let Some(ref q) = query {
                params.push(("query", q.clone()));
            }
            if let Some(ref a) = account_id {
                params.push(("accountId", a.clone()));
            }
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
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
                .get(&format!("/v1/businesses/{biz_id}/orders"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        OrderCommand::Create { data } => {
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/orders"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        OrderCommand::Update { id, data } => {
            let mut body = json!({ "id": id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/orders/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        OrderCommand::Quote { data } => {
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/orders/quote"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        OrderCommand::Checkout { data } => {
            let mut body = parse_data(data.as_deref())?;
            if body.get("businessId").is_none() {
                body["businessId"] = json!(biz_id);
            }
            let result = client
                .post(
                    &format!("/v1/businesses/{biz_id}/orders/checkout"),
                    &body,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
