use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum ProductCommand {
    /// Get a product by ID or slug
    #[command(long_about = "Fetch a single product.\n\n\
        Example:\n\
        arky product get PRODUCT_ID\n\n\
        Response shape:\n\
        {\"id\": \"...\", \"key\": \"t-shirt\", \"status\": \"active\",\n\
         \"blocks\": [{\"key\": \"title\", \"type\": \"localized_text\", \"value\": {\"en\": \"T-Shirt\"}}],\n\
         \"variants\": [{\"key\": \"default\", \"prices\": [{\"amount\": 2999, \"currency\": \"USD\", \"market\": \"us\"}],\n\
           \"inventoryLevel\": 100}],\n\
         \"filters\": [...]}")]
    Get {
        /// Product ID or slug
        id: String,
    },
    /// List products
    #[command(long_about = "List products with optional filters.\n\n\
        Examples:\n\
        arky product list\n\
        arky product list --query \"shirt\" --limit 10\n\
        arky product list --status active --sort-field createdAt --sort-direction desc\n\n\
        Response shape:\n\
        {\"data\": [{\"id\": \"...\", \"key\": \"...\", \"blocks\": [...], \"variants\": [...]}],\n\
         \"cursor\": \"...\"}")]
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long, help = "Filter: draft, active, archived")]
        status: Option<String>,
        #[arg(long)]
        sort_field: Option<String>,
        #[arg(long)]
        sort_direction: Option<String>,
    },
    /// Create a product with blocks, variants, and filters
    #[command(long_about = "Create a product.\n\n\
        Products use blocks for content and variants for purchasable options.\n\n\
        Blocks: same as nodes (text, localized_text, markdown, number, boolean,\n\
        list, map, relationship_entry, relationship_media, geo_location).\n\n\
        Variants: each variant has a key, prices, and optional inventory.\n\
          prices: [{\"currency\": \"USD\", \"market\": \"us\", \"amount\": 2999}]\n\
          amount is in cents (2999 = $29.99)\n\
          inventoryLevel: number (optional, null = unlimited)\n\n\
        Filters: categorization for product filtering.\n\
          [{\"key\": \"color\", \"values\": [\"red\", \"blue\"]}]\n\n\
        Examples:\n\
        arky product create t-shirt --data '{\n\
          \"blocks\": [\n\
            {\"key\": \"title\", \"type\": \"localized_text\", \"value\": {\"en\": \"Cool T-Shirt\"}},\n\
            {\"key\": \"description\", \"type\": \"markdown\", \"value\": {\"en\": \"# Great shirt\\nSoft cotton.\"}},\n\
            {\"key\": \"image\", \"type\": \"relationship_media\", \"value\": {\"id\": \"media_123\"}}\n\
          ],\n\
          \"variants\": [\n\
            {\"key\": \"small\", \"prices\": [{\"currency\": \"USD\", \"market\": \"us\", \"amount\": 2999}], \"inventoryLevel\": 50},\n\
            {\"key\": \"large\", \"prices\": [{\"currency\": \"USD\", \"market\": \"us\", \"amount\": 3499}], \"inventoryLevel\": 30}\n\
          ],\n\
          \"filters\": [{\"key\": \"size\", \"values\": [\"small\", \"large\"]}]\n\
        }'\n\n\
        arky product create my-product --data @product.json")]
    Create {
        /// Product key (unique within business, URL-safe)
        key: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update a product
    #[command(long_about = "Update a product by ID.\n\n\
        Blocks and variants replace the entire array. Include all you want to keep.\n\n\
        Example:\n\
        arky product update PROD_ID --data '{\"blocks\": [...], \"variants\": [...]}'\n\
        arky product update PROD_ID --data '{\"status\": \"active\"}'")]
    Update {
        /// Product ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Delete a product
    Delete {
        /// Product ID
        id: String,
    },
}

pub async fn handle(cmd: ProductCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        ProductCommand::Get { id } => {
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/products/{id}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        ProductCommand::List {
            query,
            limit,
            cursor,
            status,
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
            if let Some(ref s) = status {
                params.push(("status", s.clone()));
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
                .get(&format!("/v1/businesses/{biz_id}/products"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        ProductCommand::Create { key, data } => {
            let mut body = json!({ "key": key });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/products"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        ProductCommand::Update { id, data } => {
            let mut body = json!({ "id": id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/products/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        ProductCommand::Delete { id } => {
            let result = client
                .delete(&format!("/v1/businesses/{biz_id}/products/{id}"))
                .await?;
            crate::output::print_output(&result, format);
            crate::output::print_success("Product deleted");
        }
    }
    Ok(())
}
