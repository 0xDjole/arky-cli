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
    Required:\n\
      KEY (positional)  Product key — letters, numbers, _ and - only, max 255 chars.\n\n\
    Required (--data JSON):\n\
      slug          Localized slug: {\"en\": \"t-shirt\"}\n\
      status        \"draft\" | \"active\" | \"archived\"\n\
      audienceIds   Array of audience IDs (use [] if none)\n\
      networkIds    Array of network IDs (use [] if none)\n\
      filters       Array of filter objects (use [] if none)\n\
      blocks        Array of content blocks (same as nodes — each needs type, id, key, properties, value)\n\
      variants      Array of purchasable variants (see below)\n\n\
    Variant fields (ALL required):\n\
      key          Variant identifier, e.g. \"default\", \"small\" (required)\n\
      prices       [{\"currency\": \"usd\", \"market\": \"us\", \"amount\": 1999}] — amount in cents (required)\n\
      inventory    [{\"locationId\": \"default\", \"available\": 100, \"reserved\": 0}] (required)\n\
      attributes   Array of attribute objects (use [] if none) (required)\n\n\
    Working example (from integration tests):\n\
    arky product create t-shirt --data '{\n\
      \"slug\": {\"en\": \"t-shirt\"},\n\
      \"status\": \"active\",\n\
      \"audienceIds\": [],\n\
      \"networkIds\": [],\n\
      \"filters\": [],\n\
      \"blocks\": [\n\
        {\"type\": \"localized_text\", \"id\": \"b1\", \"key\": \"title\", \"properties\": {}, \"value\": {\"en\": \"Test Product\"}},\n\
        {\"type\": \"markdown\", \"id\": \"b2\", \"key\": \"description\", \"properties\": {}, \"value\": {\"en\": \"# Test\\nA test product\"}}\n\
      ],\n\
      \"variants\": [{\n\
        \"key\": \"default\",\n\
        \"prices\": [{\"currency\": \"usd\", \"market\": \"us\", \"amount\": 1999}],\n\
        \"inventory\": [{\"locationId\": \"default\", \"available\": 100, \"reserved\": 0}],\n\
        \"attributes\": []\n\
      }]\n\
    }'")]
    Create {
        /// Product key (unique within business, URL-safe)
        key: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update a product
    #[command(long_about = "Update a product by ID.\n\n\
        Optional (--data JSON):\n\
          blocks     Array of blocks — REPLACES entire array, include all you want to keep\n\
          variants   Array of variants — REPLACES entire array\n\
          filters    Array of filters — REPLACES entire array\n\
          status     \"draft\" | \"active\" | \"archived\"\n\n\
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
            let _ = client
                .delete(&format!("/v1/businesses/{biz_id}/products/{id}"))
                .await?;
            crate::output::print_success("Product deleted");
        }
    }
    Ok(())
}
