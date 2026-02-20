use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum AgentCommand {
    /// Get an agent by ID
    #[command(long_about = "Fetch a single AI agent.\n\n\
        Example:\n\
        arky agent get AGENT_ID\n\n\
        Response shape:\n\
        {\"id\": \"...\", \"key\": \"sales-bot\", \"status\": \"active\",\n\
         \"rolePrompt\": \"You are a helpful assistant.\",\n\
         \"provider\": {\"type\": \"deep_seek\", \"integrationId\": \"...\", \"model\": \"deepseek-chat\"},\n\
         \"toolsConfig\": [\"integration\", \"web_search\", \"read_webpage\", \"memory\"]}")]
    Get {
        /// Agent ID
        id: String,
    },
    /// List agents
    #[command(long_about = "List AI agents for the business.\n\n\
        Examples:\n\
        arky agent list\n\
        arky agent list --limit 5\n\
        arky agent list --cursor CURSOR_TOKEN\n\n\
        Response: {\"items\": [...], \"cursor\": \"...\"}")]
    List {
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Create an agent
    #[command(long_about = "Create an AI agent.\n\n\
        Required:\n\
          KEY (positional)  Agent key — unique identifier (e.g. \"sales-bot\")\n\n\
        Optional (--data JSON):\n\
          rolePrompt              System prompt defining agent behavior\n\
          status                  \"active\" (default) | \"disabled\"\n\
          provider.type            \"deep_seek\" | \"open_ai\" | \"google_gemini\" | \"perplexity\"\n\
          provider.integrationId   Integration ID for the AI provider\n\
          provider.model           Model name (e.g. \"deepseek-chat\", \"gpt-4o\", \"gemini-2.0-flash\")\n\
          toolsConfig             Array of tool names: [\"integration\", \"web_search\", \"read_webpage\", \"memory\"]\n\n\
        Available models:\n\
          deep_seek:       deepseek-chat, deepseek-reasoner\n\
          open_ai:         gpt-4o, gpt-4o-mini, o3-mini\n\
          google_gemini:   gemini-2.0-flash, gemini-2.5-pro\n\
          perplexity:      sonar, sonar-pro\n\n\
        Available tools:\n\
          integration   — Call business APIs (products, orders, services, etc.)\n\
          web_search    — Search the web via Brave Search\n\
          read_webpage  — Fetch and read webpage content\n\
          memory        — Store and recall facts about customers and business\n\n\
        Example:\n\
        arky agent create \"sales-bot\" --data '{\n\
          \"rolePrompt\": \"You are a helpful sales assistant.\",\n\
          \"provider\": { \"type\": \"deep_seek\", \"integrationId\": \"INT_ID\", \"model\": \"deepseek-chat\" },\n\
          \"toolsConfig\": [\"integration\", \"web_search\", \"read_webpage\", \"memory\"]\n\
        }'")]
    Create {
        /// Agent key
        key: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update an agent
    #[command(long_about = "Update an AI agent by ID.\n\n\
        Any field from create can be updated. Only include fields you want to change.\n\n\
        Examples:\n\
        arky agent update AGENT_ID --data '{\"status\": \"disabled\"}'\n\
        arky agent update AGENT_ID --data '{\"rolePrompt\": \"New prompt here\"}'\n\
        arky agent update AGENT_ID --data '{\"provider\": {\"model\": \"gpt-4o\"}}'\n\
        arky agent update AGENT_ID --data @agent.json")]
    Update {
        /// Agent ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Delete an agent
    #[command(long_about = "Delete an AI agent by ID.\n\n\
        Example:\n\
        arky agent delete AGENT_ID")]
    Delete {
        /// Agent ID
        id: String,
    },
    /// Run an agent with a message
    #[command(long_about = "Run an AI agent with a message and get a response.\n\n\
        The agent will use its configured tools (integration, web_search, etc.)\n\
        to look up real data before responding.\n\n\
        Examples:\n\
        arky agent run AGENT_ID --data '{\"message\": \"What services do you offer?\"}'\n\
        arky agent run AGENT_ID --data '{\"message\": \"How many products are there?\"}'\n\
        echo '{\"message\": \"Hello\"}' | arky agent run AGENT_ID --data -")]
    Run {
        /// Agent ID
        id: String,
        #[arg(long, help = "JSON data with 'message' field, or inline, @file, - for stdin")]
        data: Option<String>,
    },
    /// List agent memories
    #[command(long_about = "List memories stored by an agent.\n\n\
        Memories are automatically created during conversations. Categories:\n\
          soul     — Core personality traits and behaviors\n\
          fact     — Learned facts about customers or the business\n\
          message  — Conversation history\n\n\
        Examples:\n\
        arky agent memories AGENT_ID\n\
        arky agent memories AGENT_ID --category fact\n\
        arky agent memories AGENT_ID --category message --limit 10")]
    Memories {
        /// Agent ID
        id: String,
        #[arg(long, help = "Filter: soul, message, fact")]
        category: Option<String>,
        #[arg(long, default_value = "100")]
        limit: u32,
    },
    /// Delete a specific memory
    #[command(name = "delete-memory", long_about = "Delete a specific memory from an agent.\n\n\
        Example:\n\
        arky agent delete-memory AGENT_ID MEMORY_ID")]
    DeleteMemory {
        /// Agent ID
        id: String,
        /// Memory ID
        memory_id: String,
    },
}

pub async fn handle(cmd: AgentCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        AgentCommand::Get { id } => {
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/agents/{id}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        AgentCommand::List { limit, cursor } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/agents"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        AgentCommand::Create { key, data } => {
            let mut body = json!({ "key": key, "businessId": biz_id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/agents"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        AgentCommand::Update { id, data } => {
            let mut body = json!({ "id": id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/agents/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        AgentCommand::Delete { id } => {
            let _ = client
                .delete(&format!("/v1/businesses/{biz_id}/agents/{id}"))
                .await?;
            crate::output::print_success("Agent deleted");
        }
        AgentCommand::Run { id, data } => {
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/agents/{id}/run"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        AgentCommand::Memories {
            id,
            category,
            limit,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref c) = category {
                params.push(("category", c.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(
                    &format!("/v1/businesses/{biz_id}/agents/{id}/memories"),
                    &params_ref,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        AgentCommand::DeleteMemory { id, memory_id } => {
            let _ = client
                .delete(&format!(
                    "/v1/businesses/{biz_id}/agents/{id}/memories/{memory_id}"
                ))
                .await?;
            crate::output::print_success("Memory deleted");
        }
    }
    Ok(())
}
