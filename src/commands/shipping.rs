use crate::client::ArkyClient;
use crate::commands::parse_data;
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ShippingCommand {
    /// Get shipping rates for an order
    #[command(long_about = "Get available shipping rates for an order.\n\n\
        Requires shipping provider configuration. Returns available rates\n\
        from carriers with prices, estimated delivery times.\n\n\
        Address fields: name, street1, street2 (optional), city, state,\n\
        postalCode, country (ISO 2-letter).\n\n\
        Parcel fields: length, width, height, weight,\n\
        distanceUnit (in|cm), massUnit (oz|g|lb|kg).\n\n\
        Example:\n\
        arky shipping rates ORDER_ID --data '{\n\
          \"shippingProviderId\": \"integration_123\",\n\
          \"fromAddress\": {\n\
            \"name\": \"Warehouse\", \"street1\": \"123 Main St\",\n\
            \"city\": \"NYC\", \"state\": \"NY\", \"postalCode\": \"10001\", \"country\": \"US\"\n\
          },\n\
          \"toAddress\": {\n\
            \"name\": \"Customer\", \"street1\": \"456 Oak Ave\",\n\
            \"city\": \"LA\", \"state\": \"CA\", \"postalCode\": \"90001\", \"country\": \"US\"\n\
          },\n\
          \"parcel\": {\n\
            \"length\": 10, \"width\": 8, \"height\": 4, \"weight\": 16,\n\
            \"distanceUnit\": \"in\", \"massUnit\": \"oz\"\n\
          }\n\
        }'\n\n\
        Response shape:\n\
        [{\"rateId\": \"rate_abc\", \"carrier\": \"usps\", \"service\": \"usps_priority\",\n\
          \"amount\": 795, \"currency\": \"USD\", \"estimatedDays\": 3}]")]
    Rates {
        /// Order ID
        order_id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Ship an order: create shipment + purchase label
    #[command(long_about = "Create a shipment and purchase a shipping label.\n\n\
        Use a rate ID from `arky shipping rates` to select the carrier/service.\n\n\
        Example:\n\
        arky shipping ship ORDER_ID --data '{\n\
          \"rateId\": \"rate_abc\",\n\
          \"carrier\": \"usps\",\n\
          \"service\": \"usps_priority\",\n\
          \"locationId\": \"loc_123\",\n\
          \"lines\": [{\"orderItemId\": \"item_1\", \"quantity\": 1}]\n\
        }'\n\n\
        Response includes: trackingNumber, labelUrl, carrier, service.")]
    Ship {
        /// Order ID
        order_id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
}

pub async fn handle(cmd: ShippingCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        ShippingCommand::Rates { order_id, data } => {
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(
                    &format!("/v1/businesses/{biz_id}/orders/{order_id}/shipping/rates"),
                    &body,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        ShippingCommand::Ship { order_id, data } => {
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(
                    &format!("/v1/businesses/{biz_id}/orders/{order_id}/ship"),
                    &body,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
