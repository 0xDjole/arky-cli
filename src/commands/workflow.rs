use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum WorkflowCommand {
    /// Get a workflow by ID
    #[command(long_about = "Fetch a single workflow with all its nodes.\n\n\
        Example:\n\
        arky workflow get WORKFLOW_ID\n\n\
        Response shape:\n\
        {\"id\": \"...\", \"key\": \"...\", \"status\": \"active\",\n\
         \"nodes\": {\"trigger\": {\"type\": \"trigger\"}, \"fetch\": {\"type\": \"http\", ...}},\n\
         \"triggerSecret\": \"sec_abc\", \"businessId\": \"...\"}")]
    Get {
        /// Workflow ID
        id: String,
    },
    /// List workflows
    #[command(long_about = "List workflows with optional filters.\n\n\
        Examples:\n\
        arky workflow list\n\
        arky workflow list --limit 5 --statuses active\n\n\
        Response shape:\n\
        {\"data\": [{\"id\": \"...\", \"key\": \"...\", \"status\": \"...\"}], \"cursor\": \"...\"}")]
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long, help = "Comma-separated: draft,active,archived")]
        statuses: Option<String>,
    },
    /// Create a workflow
    #[command(long_about = "Create a workflow with DAG-based node execution.\n\n\
        Required:\n\
          KEY (positional)  Workflow key — letters, numbers, _ and - only, max 255 chars.\n\
          nodes             Object of named nodes (--data). Must include exactly one trigger node.\n\n\
        Optional (--data JSON):\n\
          status     \"draft\" (default) | \"active\" | \"archived\"\n\
          schedule   Cron expression for scheduled triggers\n\n\
        Workflows are directed acyclic graphs (DAGs) of nodes.\n\n\
        Node types:\n\
          trigger   Entry point. Receives external data when triggered.\n\
                    Fields: event, schema (Block[] for input validation), delayMs\n\n\
          http      Makes an HTTP request.\n\
                    Required: method (get|post|put|delete), url, headers (object, e.g. {}),\n\
                    timeoutMs (integer, e.g. 30000), delayMs (integer, e.g. 0)\n\
                    retries (integer, e.g. 0), retryDelayMs (integer, e.g. 0)\n\
                    Optional: body (supports {{expr}} interpolation), integrationId\n\n\
          switch    Conditional branching. Evaluates rules in order.\n\
                    Fields: rules [{condition}] — raw JS expressions.\n\
                    Outputs: \"0\", \"1\", ... (rule index) and \"default\".\n\n\
          transform Data transformation.\n\
                    Fields: code — raw JS that returns a value.\n\
                    Access upstream nodes by name: trigger.data, fetch.status\n\n\
          loop      Iterate over arrays.\n\
                    Fields: expression — JS that returns an array.\n\
                    Outputs: \"each\" (current item), \"default\" (collected results).\n\
                    Body nodes connect via edges, back-edges return to loop.\n\n\
        Connections (edges):\n\
          Each node has edges: [{\"node\": \"source_node_id\", \"output\": \"output_name\"}]\n\
          This defines WHERE this node gets its input FROM.\n\
          Common outputs: \"default\" (main output), \"each\" (loop item)\n\
          Switch outputs: \"0\", \"1\", \"2\", ... (rule index), \"default\"\n\n\
        Expression engine (JS via boa runtime):\n\
          Switch conditions: raw JS — trigger.status === \"active\"\n\
          Transform code: raw JS — fetch.data.map(x => x.name)\n\
          HTTP body: {{expression}} template interpolation\n\
          Loop expression: raw JS returning array — trigger.items\n\n\
        Example — simple fetch + transform:\n\
        arky workflow create my-workflow --data '{\n\
          \"nodes\": {\n\
            \"trigger\": {\"type\": \"trigger\"},\n\
            \"fetch\": {\n\
              \"type\": \"http\", \"method\": \"get\",\n\
              \"url\": \"https://api.example.com/data\",\n\
              \"headers\": {}, \"timeoutMs\": 30000,\n\
              \"delayMs\": 0, \"retries\": 0, \"retryDelayMs\": 0,\n\
              \"edges\": [{\"node\": \"trigger\", \"output\": \"default\"}]\n\
            },\n\
            \"process\": {\n\
              \"type\": \"transform\",\n\
              \"code\": \"fetch.data.map(x => x.name)\",\n\
              \"edges\": [{\"node\": \"fetch\", \"output\": \"default\"}]\n\
            }\n\
          }\n\
        }'\n\n\
        Example — with switch:\n\
        arky workflow create branching --data '{\n\
          \"nodes\": {\n\
            \"trigger\": {\"type\": \"trigger\"},\n\
            \"check\": {\n\
              \"type\": \"switch\",\n\
              \"rules\": [{\"condition\": \"trigger.type === \\\"premium\\\"\"}],\n\
              \"edges\": [{\"node\": \"trigger\", \"output\": \"default\"}]\n\
            },\n\
            \"premium_action\": {\n\
              \"type\": \"http\", \"method\": \"post\",\n\
              \"url\": \"https://api.example.com/premium\",\n\
              \"headers\": {\"Content-Type\": \"application/json\"},\n\
              \"timeoutMs\": 30000, \"delayMs\": 0, \"retries\": 0, \"retryDelayMs\": 0,\n\
              \"edges\": [{\"node\": \"check\", \"output\": \"0\"}]\n\
            },\n\
            \"default_action\": {\n\
              \"type\": \"http\", \"method\": \"post\",\n\
              \"url\": \"https://api.example.com/basic\",\n\
              \"headers\": {\"Content-Type\": \"application/json\"},\n\
              \"timeoutMs\": 30000, \"delayMs\": 0, \"retries\": 0, \"retryDelayMs\": 0,\n\
              \"edges\": [{\"node\": \"check\", \"output\": \"default\"}]\n\
            }\n\
          }\n\
        }'\n\n\
        Minimal working example (from integration tests):\n\
        arky workflow create my-workflow --data '{\n\
          \"status\": \"draft\",\n\
          \"nodes\": {\n\
            \"trigger\": {\"type\": \"trigger\"},\n\
            \"process\": {\n\
              \"type\": \"transform\",\n\
              \"code\": \"trigger\",\n\
              \"edges\": [{\"node\": \"trigger\", \"output\": \"default\"}]\n\
            }\n\
          }\n\
        }'")]
    Create {
        /// Workflow key (unique within business)
        key: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Update a workflow
    #[command(long_about = "Update a workflow by ID.\n\n\
        Optional (--data JSON):\n\
          nodes      Node map — REPLACES entirely, include all nodes (must have 1 trigger)\n\
          status     \"draft\" | \"active\" | \"archived\"\n\
          schedule   Cron expression for scheduled triggers\n\n\
        Example:\n\
        arky workflow update WF_ID --data '{\"nodes\": {...}, \"status\": \"active\"}'")]
    Update {
        /// Workflow ID
        id: String,
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Delete a workflow
    Delete {
        /// Workflow ID
        id: String,
    },
    /// Trigger a workflow by its secret
    #[command(long_about = "Trigger a workflow execution via its trigger secret.\n\n\
        The trigger secret is returned when creating a workflow.\n\
        You can find it via `arky workflow get WORKFLOW_ID` in the triggerSecret field.\n\n\
        Pass input data via --data to make it available as `trigger` in expressions.\n\n\
        Examples:\n\
        arky workflow trigger sec_abc123\n\
        arky workflow trigger sec_abc123 --data '{\"email\": \"user@example.com\", \"type\": \"welcome\"}'")]
    Trigger {
        /// Workflow trigger secret
        secret: String,
        /// JSON payload to pass as trigger input
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// List executions of a workflow
    #[command(long_about = "List past executions of a workflow.\n\n\
        Statuses: pending, running, completed, failed.\n\n\
        Example:\n\
        arky workflow executions WORKFLOW_ID --limit 5 --status completed")]
    Executions {
        /// Workflow ID
        workflow_id: String,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
        #[arg(long, help = "Filter: pending, running, completed, failed")]
        status: Option<String>,
    },
    /// Get a specific execution
    #[command(long_about = "Fetch details of a specific workflow execution.\n\n\
        Shows status, node results, errors, and timing.\n\n\
        Example:\n\
        arky workflow execution WORKFLOW_ID EXECUTION_ID")]
    Execution {
        /// Workflow ID
        workflow_id: String,
        /// Execution ID
        execution_id: String,
    },
}

pub async fn handle(cmd: WorkflowCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    let biz_id = client.require_business_id()?;

    match cmd {
        WorkflowCommand::Get { id } => {
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/workflows/{id}"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        WorkflowCommand::List {
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
                .get(&format!("/v1/businesses/{biz_id}/workflows"), &params_ref)
                .await?;
            crate::output::print_output(&result, format);
        }
        WorkflowCommand::Create { key, data } => {
            let mut body = json!({ "key": key, "businessId": biz_id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/workflows"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        WorkflowCommand::Update { id, data } => {
            let mut body = json!({ "id": id });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/workflows/{id}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        WorkflowCommand::Delete { id } => {
            let _ = client
                .delete(&format!("/v1/businesses/{biz_id}/workflows/{id}"))
                .await?;
            crate::output::print_success("Workflow deleted");
        }
        WorkflowCommand::Trigger { secret, data } => {
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(&format!("/v1/workflows/trigger/{secret}"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        WorkflowCommand::Executions {
            workflow_id,
            limit,
            cursor,
            status,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            if let Some(ref s) = status {
                params.push(("status", s.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client
                .get(
                    &format!("/v1/businesses/{biz_id}/workflows/{workflow_id}/executions"),
                    &params_ref,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        WorkflowCommand::Execution {
            workflow_id,
            execution_id,
        } => {
            let result = client
                .get(
                    &format!(
                        "/v1/businesses/{biz_id}/workflows/{workflow_id}/executions/{execution_id}"
                    ),
                    &[],
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
