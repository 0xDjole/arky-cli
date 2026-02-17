use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum BookingCommand {
    /// Get a booking by ID
    #[command(long_about = "Fetch a single booking with all details.\n\n\
        Example:\n\
        arky booking get BOOKING_ID\n\n\
        Response includes: id, status, serviceId, providerId, from, to,\n\
        accountId, items, totals.")]
    Get {
        /// Booking ID
        id: String,
    },
    /// Search bookings
    #[command(long_about = "Search and filter bookings.\n\n\
        Time range filters use epoch seconds.\n\
        Statuses: pending, confirmed, cancelled, completed.\n\n\
        Examples:\n\
        arky booking list\n\
        arky booking list --service-id SVC_ID --from 1771405200 --to 1771410600\n\
        arky booking list --provider-id PROV_ID --status confirmed\n\
        arky booking list --account-id ACC_ID")]
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        service_id: Option<String>,
        #[arg(long)]
        provider_id: Option<String>,
        #[arg(long)]
        account_id: Option<String>,
        #[arg(long, help = "Start timestamp (epoch seconds)")]
        from: Option<String>,
        #[arg(long, help = "End timestamp (epoch seconds)")]
        to: Option<String>,
        #[arg(long, help = "Filter: pending, confirmed, cancelled, completed")]
        status: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Create a booking directly (admin use)
    #[command(long_about = "Create a booking directly (bypasses checkout flow).\n\n\
        For customer-facing booking with payment, use `arky booking checkout`.\n\n\
        Required (--data JSON):\n\
          items    Array of booking items (at least one, see fields below)\n\n\
        Optional:\n\
          market           Market identifier (defaults to \"default\")\n\
          paymentMethodId  Payment method ID\n\
          promoCode        Promo code string\n\n\
        Item fields (ALL required):\n\
          serviceId   Service ID\n\
          providerId  Provider ID\n\
          from        Start time as EPOCH SECONDS (NOT milliseconds!)\n\
          to          End time as EPOCH SECONDS (NOT milliseconds!)\n\
          blocks      Array of content blocks (use [] if none)\n\n\
        IMPORTANT: from/to are epoch SECONDS. Duration (to - from) must be\n\
        evenly divisible by the service's duration unit.\n\
        Do NOT pass id or price on items — the server auto-generates those.\n\n\
        Working example:\n\
        arky booking create --data '{\n\
          \"items\": [{\n\
            \"serviceId\": \"SVC_ID\",\n\
            \"providerId\": \"PROV_ID\",\n\
            \"from\": 1771405200,\n\
            \"to\": 1771410600,\n\
            \"blocks\": []\n\
          }],\n\
          \"market\": \"us\"\n\
        }'")]
    Create {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update a booking
    #[command(long_about = "Update a booking (e.g., change status, reschedule).\n\n\
        Optional (--data JSON):\n\
          status   \"pending\" | \"confirmed\" | \"cancelled\" | \"completed\"\n\n\
        Example:\n\
        arky booking update BOOKING_ID --data '{\"status\": \"cancelled\"}'")]
    Update {
        /// Booking ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Get a booking price quote
    #[command(long_about = "Calculate prices for a booking without creating it.\n\n\
        Use to preview pricing, availability, and totals before creating.\n\n\
        Required (--data JSON):\n\
          items    Array of booking items (serviceId, providerId, from, to)\n\n\
        Optional:\n\
          market   Market identifier (defaults to \"default\")\n\n\
        Item fields (ALL required):\n\
          serviceId   Service ID\n\
          providerId  Provider ID\n\
          from        Start time as EPOCH SECONDS (NOT milliseconds!)\n\
          to          End time as EPOCH SECONDS (NOT milliseconds!)\n\n\
        Working example:\n\
        arky booking quote --data '{\n\
          \"items\": [{\n\
            \"serviceId\": \"SVC_ID\",\n\
            \"providerId\": \"PROV_ID\",\n\
            \"from\": 1771405200,\n\
            \"to\": 1771410600\n\
          }],\n\
          \"market\": \"us\"\n\
        }'")]
    Quote {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Checkout: create booking and process payment
    #[command(long_about = "Create a booking with payment in one step.\n\n\
        This is the primary booking flow for customers.\n\n\
        Required (--data JSON):\n\
          items    Array of booking items (see fields below)\n\n\
        Optional:\n\
          market           Market identifier (defaults to \"default\")\n\
          paymentMethodId  Payment method ID for charging\n\n\
        Item fields (ALL required):\n\
          serviceId   Service ID\n\
          providerId  Provider ID\n\
          from        Start time as EPOCH SECONDS (NOT milliseconds!)\n\
          to          End time as EPOCH SECONDS (NOT milliseconds!)\n\
          blocks      Array of content blocks (use [] if none)\n\n\
        Working example:\n\
        arky booking checkout --data '{\n\
          \"items\": [{\n\
            \"serviceId\": \"SVC_ID\",\n\
            \"providerId\": \"PROV_ID\",\n\
            \"from\": 1771405200,\n\
            \"to\": 1771410600,\n\
            \"blocks\": []\n\
          }],\n\
          \"market\": \"us\"\n\
        }'")]
    Checkout {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
}

pub async fn handle(cmd: BookingCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        BookingCommand::Get { id } => {
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/bookings/{id}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        BookingCommand::List {
            query,
            service_id,
            provider_id,
            account_id,
            from,
            to,
            status,
            limit,
            cursor,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref q) = query {
                params.push(("query", q.clone()));
            }
            if let Some(ref s) = service_id {
                params.push(("serviceIds", s.clone()));
            }
            if let Some(ref p) = provider_id {
                params.push(("providerIds", p.clone()));
            }
            if let Some(ref a) = account_id {
                params.push(("accountId", a.clone()));
            }
            if let Some(ref f) = from {
                params.push(("from", f.clone()));
            }
            if let Some(ref t) = to {
                params.push(("to", t.clone()));
            }
            if let Some(ref st) = status {
                params.push(("status", st.clone()));
            }
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/bookings"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        BookingCommand::Create { data } => {
            let mut body = parse_data(data.as_deref())?;
            if body.get("market").is_none() {
                body["market"] = json!("default");
            }
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/bookings"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        BookingCommand::Update { id, data } => {
            let overlay = parse_data(data.as_deref())?;
            let mut body = json!({});
            merge_data(&mut body, overlay);
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/bookings/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        BookingCommand::Quote { data } => {
            let mut body = parse_data(data.as_deref())?;
            if body.get("market").is_none() {
                body["market"] = json!("default");
            }
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/bookings/quote"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        BookingCommand::Checkout { data } => {
            let mut body = parse_data(data.as_deref())?;
            if body.get("market").is_none() {
                body["market"] = json!("default");
            }
            let result = client
                .post(
                    &format!("/v1/businesses/{biz_id}/bookings/checkout"),
                    &body,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
