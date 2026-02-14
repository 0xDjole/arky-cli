use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum NodeCommand {
    /// Get a content node by ID, slug, or key
    #[command(long_about = "Fetch a single content node.\n\n\
        Accepts node ID, slug, or key as the identifier.\n\n\
        Example:\n\
        arky node get NODE_ID\n\
        arky node get my-blog-post\n\n\
        Response shape:\n\
        {\"id\": \"...\", \"key\": \"my-post\", \"type\": \"blog\", \"status\": \"active\",\n\
         \"blocks\": [\n\
           {\"key\": \"title\", \"type\": \"localized_text\", \"value\": {\"en\": \"Hello\"}},\n\
           {\"key\": \"body\", \"type\": \"markdown\", \"value\": {\"en\": \"# Content...\"}}\n\
         ],\n\
         \"parentId\": null, \"businessId\": \"...\", \"createdAt\": \"...\"}")]
    Get {
        /// Node ID, slug, or key
        id: String,
    },
    /// List content nodes
    #[command(long_about = "List content nodes with optional filters.\n\n\
        Nodes are the CMS building block: pages, blog posts, newsletters, etc.\n\
        Each node has a type, key, status, and an array of blocks for content.\n\n\
        Filter by type to get specific content kinds.\n\
        Statuses: draft, active, archived.\n\n\
        Examples:\n\
        arky node list\n\
        arky node list --type blog --limit 10\n\
        arky node list --query \"hello\" --statuses active\n\
        arky node list --parent-id PARENT_NODE_ID\n\
        arky node list --sort-field createdAt --sort-direction desc\n\n\
        Response shape:\n\
        {\"data\": [{\"id\": \"...\", \"key\": \"...\", \"type\": \"...\", \"status\": \"...\",\n\
          \"blocks\": [...]}], \"cursor\": \"next_page_cursor\"}")]
    List {
        #[arg(long, help = "Filter by node type (e.g., blog, page, newsletter)")]
        r#type: Option<String>,
        #[arg(long)]
        query: Option<String>,
        #[arg(long)]
        key: Option<String>,
        #[arg(long)]
        parent_id: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long, help = "Comma-separated: draft,active,archived")]
        statuses: Option<String>,
        #[arg(long)]
        sort_field: Option<String>,
        #[arg(long)]
        sort_direction: Option<String>,
    },
    /// Create a content node
    #[command(long_about = "Create a content node with blocks.\n\n\
        Blocks are the core content model. Each block has:\n\
          key:   unique identifier within the node\n\
          type:  one of the block types below\n\
          value: the content (type-dependent)\n\
          properties: optional metadata (label, variant, etc.)\n\n\
        Block types:\n\
          text              - simple string: \"Hello world\"\n\
          localized_text    - per-locale: {\"en\": \"Hello\", \"bs\": \"Zdravo\"}\n\
          markdown          - localized markdown: {\"en\": \"# Title\\nBody\"}\n\
          number            - numeric: 42 (also timestamps as epoch ms for dates)\n\
          boolean           - true or false\n\
          list              - array of sub-block objects: [{\"key\":\"item\",\"type\":\"text\",\"value\":\"A\"}]\n\
          map               - key-value sub-blocks\n\
          relationship_entry - reference to another entity: {\"id\": \"node_123\"}\n\
          relationship_media - media reference: {\"id\": \"media_123\", \"resolutions\": {...}}\n\
          geo_location       - coordinates: {\"coordinates\": {\"lat\": 43.85, \"lon\": 18.41}}\n\n\
        Data input:\n\
          --data '{...}'    Inline JSON\n\
          --data @file.json Read from file\n\
          --data -          Read from stdin\n\n\
        Examples:\n\
        arky node create blog-post --data '{\n\
          \"blocks\": [\n\
            {\"key\": \"title\", \"type\": \"localized_text\", \"value\": {\"en\": \"My Post\"}},\n\
            {\"key\": \"body\", \"type\": \"markdown\", \"value\": {\"en\": \"# Hello\\nWorld\"}},\n\
            {\"key\": \"image\", \"type\": \"relationship_media\", \"value\": {\"id\": \"media_abc\"}},\n\
            {\"key\": \"published\", \"type\": \"boolean\", \"value\": true}\n\
          ]\n\
        }'\n\n\
        arky node create my-page --data @content.json\n\
        echo '{\"blocks\":[...]}' | arky node create my-page --data -")]
    Create {
        /// Node key (unique within business, URL-safe)
        key: String,
        #[arg(long)]
        parent_id: Option<String>,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update a content node
    #[command(long_about = "Update a content node.\n\n\
        Pass blocks via --data. Blocks replace the entire blocks array.\n\
        Include all blocks you want to keep, not just changed ones.\n\n\
        Block types:\n\
          text              - simple string: \"Hello world\"\n\
          localized_text    - {\"en\": \"English\", \"bs\": \"Bosnian\"}\n\
          markdown          - {\"en\": \"# Title\\nContent\"}\n\
          number            - numeric value (also timestamps as epoch ms)\n\
          boolean           - true/false\n\
          list              - array of sub-block objects\n\
          map               - key-value sub-blocks\n\
          relationship_entry - {\"id\": \"node_123\"}\n\
          relationship_media - {\"id\": \"media_123\", \"resolutions\": {...}}\n\
          geo_location       - {\"coordinates\": {\"lat\": 43.85, \"lon\": 18.41}}\n\n\
        Examples:\n\
        arky node update NODE_ID --data '{\"blocks\":[{\"key\":\"title\",\"type\":\"localized_text\",\"value\":{\"en\":\"Updated\"}}]}'\n\
        arky node update NODE_ID --data '{\"status\": \"active\"}'")]
    Update {
        /// Node ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Delete a content node
    Delete {
        /// Node ID
        id: String,
    },
    /// Get children of a content node
    #[command(long_about = "List child nodes of a parent node.\n\n\
        Nodes can be hierarchical (parent-child). Use this to navigate the tree.\n\n\
        Example:\n\
        arky node children PARENT_NODE_ID --limit 10")]
    Children {
        /// Parent node ID
        id: String,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
    },
}

pub async fn handle(cmd: NodeCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        NodeCommand::Get { id } => {
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/nodes/{id}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        NodeCommand::List {
            r#type,
            query,
            key,
            parent_id,
            limit,
            cursor,
            statuses,
            sort_field,
            sort_direction,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref t) = r#type {
                params.push(("type", t.clone()));
            }
            if let Some(ref q) = query {
                params.push(("query", q.clone()));
            }
            if let Some(ref k) = key {
                params.push(("key", k.clone()));
            }
            if let Some(ref p) = parent_id {
                params.push(("parentId", p.clone()));
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
                .get(&format!("/v1/businesses/{biz_id}/nodes"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        NodeCommand::Create {
            key,
            parent_id,
            data,
        } => {
            let mut body = json!({ "key": key });
            if let Some(pid) = parent_id {
                body["parentId"] = json!(pid);
            }
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/nodes"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        NodeCommand::Update { id, data } => {
            let mut body = json!({ "id": id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/nodes/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        NodeCommand::Delete { id } => {
            let result = client
                .delete(&format!("/v1/businesses/{biz_id}/nodes/{id}"))
                .await?;
            crate::output::print_output(&result, format);
            crate::output::print_success("Node deleted");
        }
        NodeCommand::Children { id, limit, cursor } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(
                    &format!("/v1/businesses/{biz_id}/nodes/{id}/children"),
                    &params_ref,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
