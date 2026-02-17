use crate::client::ArkyClient;
use crate::commands::{merge_data, parse_data};
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum BusinessCommand {
    /// Get the current business details
    #[command(long_about = "Fetch the business identified by --business-id or ARKY_BUSINESS_ID.\n\n\
        Example:\n\
        arky business get\n\n\
        Response shape:\n\
        {\"id\": \"biz_123\", \"key\": \"my-shop\", \"name\": \"My Shop\",\n\
         \"blocks\": [...], \"status\": \"active\", \"createdAt\": \"...\"}")]
    Get,
    /// List all businesses
    #[command(long_about = "List businesses accessible to the current account.\n\n\
        Examples:\n\
        arky business list\n\
        arky business list --limit 5\n\
        arky business list --query \"shop\"\n\n\
        Response shape:\n\
        {\"data\": [{\"id\": \"...\", \"key\": \"...\", \"name\": \"...\"}], \"cursor\": \"...\"}")]
    List {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Create a new business
    #[command(long_about = "Create a new business.\n\n\
        Required:\n\
          KEY (positional)  Business key — unique, URL-safe (lowercase, hyphens)\n\n\
        Required (--data JSON):\n\
          status     \"active\" | \"draft\" | \"archived\"\n\
          timezone   IANA timezone string (e.g. \"UTC\", \"America/New_York\")\n\
          configs    Business configuration object (see below)\n\n\
        Configs object:\n\
          currencies        [{\"code\": \"usd\", \"symbol\": \"$\"}]\n\
          markets           [{\"id\": \"us\", \"name\": \"United States\", \"currencies\": [\"usd\"], ...}]\n\
          locations         [] (warehouse/pickup locations)\n\
          paymentProviders  [] (Stripe, etc.)\n\
          shippingProviders [] (shipping integrations)\n\
          emails            {\"billing\": \"you@example.com\", \"support\": \"you@example.com\"}\n\n\
        Working example (from integration tests):\n\
        arky business create my-shop --data '{\n\
          \"status\": \"active\",\n\
          \"timezone\": \"UTC\",\n\
          \"configs\": {\n\
            \"currencies\": [{\"code\": \"usd\", \"symbol\": \"$\", \"decimals\": 2}],\n\
            \"markets\": [{\n\
              \"id\": \"us\",\n\
              \"name\": \"United States\",\n\
              \"currencies\": [\"usd\"],\n\
              \"countries\": [\"US\"],\n\
              \"languages\": [\"en\"],\n\
              \"defaultLanguage\": \"en\"\n\
            }],\n\
            \"locations\": [],\n\
            \"paymentProviders\": [],\n\
            \"shippingProviders\": [],\n\
            \"emails\": {\"billing\": \"test@example.com\", \"support\": \"test@example.com\"}\n\
          }\n\
        }'")]
    Create {
        /// Business key (unique identifier)
        key: String,
        /// JSON data for the business
        #[arg(long)]
        data: Option<String>,
    },
    /// Update a business
    #[command(long_about = "Update a business by ID.\n\n\
        Example:\n\
        arky business update BIZ_ID --data '{\"name\": \"New Name\"}'")]
    Update {
        /// Business ID
        id: String,
        /// JSON data to update
        #[arg(long)]
        data: Option<String>,
    },
    /// Delete a business
    Delete {
        /// Business ID
        id: String,
    },
    /// Get parent businesses in hierarchy
    #[command(long_about = "Fetch parent businesses in the hierarchy.\n\n\
        Example:\n\
        arky business parents")]
    Parents,
    /// Trigger a rebuild/deploy of the business
    #[command(name = "trigger-builds", long_about = "Trigger a rebuild/deploy for the business.\n\n\
        Example:\n\
        arky business trigger-builds")]
    TriggerBuilds,
    /// List available subscription plans
    Plans,
    /// Get current subscription details
    Subscription,
    /// Subscribe to a plan (creates Stripe checkout)
    #[command(long_about = "Subscribe the business to a plan.\n\n\
        Required (--data JSON):\n\
          planId      Plan ID to subscribe to\n\
          successUrl  Redirect URL on successful payment\n\
          cancelUrl   Redirect URL on cancelled payment\n\n\
        Example:\n\
        arky business subscribe --data '{\"planId\": \"plan_123\", \"successUrl\": \"https://...\", \"cancelUrl\": \"https://...\"}'")]
    Subscribe {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Create a Stripe billing portal session
    #[command(long_about = "Create a Stripe billing portal session for managing subscription.\n\n\
        Required (--data JSON):\n\
          returnUrl   URL to return to after portal session\n\n\
        Example:\n\
        arky business portal --data '{\"returnUrl\": \"https://...\"}'")]
    Portal {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Invite a user to the business team
    #[command(long_about = "Send an invitation to join the business.\n\n\
        Required:\n\
          --email   Email address of the person to invite\n\n\
        Optional:\n\
          --role    Role to assign (default: member)\n\n\
        Example:\n\
        arky business invite --email user@example.com --role admin")]
    Invite {
        #[arg(long)]
        email: String,
        #[arg(long)]
        role: Option<String>,
    },
    /// Remove a member from the business team
    #[command(name = "remove-member", long_about = "Remove a team member from the business.\n\n\
        Required:\n\
          --account-id   Account ID of the member to remove\n\n\
        Example:\n\
        arky business remove-member --account-id ACC_ID")]
    RemoveMember {
        #[arg(long)]
        account_id: String,
    },
    /// Accept or reject a business invitation
    #[command(name = "handle-invitation", long_about = "Accept or reject an invitation to join a business.\n\n\
        Required:\n\
          --token    Invitation token\n\
          --action   \"accept\" or \"reject\"\n\n\
        Example:\n\
        arky business handle-invitation --token INV_TOKEN --action accept")]
    HandleInvitation {
        #[arg(long)]
        token: String,
        #[arg(long)]
        action: String,
    },
    /// Test a webhook configuration
    #[command(name = "test-webhook", long_about = "Send a test event to a webhook URL.\n\n\
        Required (--data JSON):\n\
          url      Webhook URL to test\n\
          events   Array of event types to include\n\n\
        Example:\n\
        arky business test-webhook --data '{\"url\": \"https://...\", \"events\": [\"order.paid\"]}'")]
    TestWebhook {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Process a refund
    #[command(long_about = "Process a refund for an order or booking.\n\n\
        Required (--data JSON):\n\
          entity   Entity ID (order or booking ID)\n\
          amount   Refund amount in cents\n\n\
        Example:\n\
        arky business refund --data '{\"entity\": \"order_123\", \"amount\": 2999}'")]
    Refund {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Connect an OAuth provider
    #[command(name = "oauth-connect", long_about = "Connect an OAuth provider to the business.\n\n\
        Required (--data JSON):\n\
          provider     OAuth provider name (e.g. \"google\", \"stripe\")\n\
          code         Authorization code from OAuth flow\n\
          redirectUri  Redirect URI used in the OAuth flow\n\n\
        Example:\n\
        arky business oauth-connect --data '{\"provider\": \"google\", \"code\": \"AUTH_CODE\", \"redirectUri\": \"https://...\"}'")]
    OauthConnect {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Disconnect an OAuth provider
    #[command(name = "oauth-disconnect", long_about = "Disconnect an OAuth provider from the business.\n\n\
        Required:\n\
          --provider   OAuth provider name to disconnect\n\n\
        Example:\n\
        arky business oauth-disconnect --provider google")]
    OauthDisconnect {
        #[arg(long)]
        provider: String,
    },
}

pub async fn handle(cmd: BusinessCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    match cmd {
        BusinessCommand::Get => {
            let biz_id = client.require_business_id()?;
            let result = client.get(&format!("/v1/businesses/{biz_id}"), &[]).await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::List {
            query,
            limit,
            cursor,
        } => {
            let mut params: Vec<(&str, String)> = vec![("limit", limit.to_string())];
            if let Some(ref q) = query {
                params.push(("query", q.clone()));
            }
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client.get("/v1/businesses", &params_ref).await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Create { key, data } => {
            let mut body = json!({ "key": key });
            let overlay = parse_data(data.as_deref())?;
            merge_data(&mut body, overlay);
            let result = client.post("/v1/businesses", &body).await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Update { id, data } => {
            let overlay = parse_data(data.as_deref())?;
            let mut body = json!({ "id": id });
            merge_data(&mut body, overlay);
            let result = client.put(&format!("/v1/businesses/{id}"), &body).await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Delete { id } => {
            let _ = client.delete(&format!("/v1/businesses/{id}")).await?;
            crate::output::print_success("Business deleted");
        }
        BusinessCommand::Parents => {
            let biz_id = client.require_business_id()?;
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/parents"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::TriggerBuilds => {
            let biz_id = client.require_business_id()?;
            let _ = client
                .post(&format!("/v1/businesses/{biz_id}/trigger-builds"), &json!({}))
                .await?;
            crate::output::print_success("Build triggered");
        }
        BusinessCommand::Plans => {
            let result = client.get("/v1/businesses/plans", &[]).await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Subscription => {
            let biz_id = client.require_business_id()?;
            let result = client
                .get(&format!("/v1/businesses/{biz_id}/subscription"), &[])
                .await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Subscribe { data } => {
            let biz_id = client.require_business_id()?;
            let body = parse_data(data.as_deref())?;
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/subscribe"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Portal { data } => {
            let biz_id = client.require_business_id()?;
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(
                    &format!("/v1/businesses/{biz_id}/subscription/portal"),
                    &body,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Invite { email, role } => {
            let biz_id = client.require_business_id()?;
            let mut body = json!({ "email": email });
            if let Some(r) = role {
                body["role"] = json!(r);
            }
            let _ = client
                .post(&format!("/v1/businesses/{biz_id}/invitation"), &body)
                .await?;
            crate::output::print_success(&format!("Invitation sent to {email}"));
        }
        BusinessCommand::RemoveMember { account_id } => {
            let biz_id = client.require_business_id()?;
            let _ = client
                .delete(&format!(
                    "/v1/businesses/{biz_id}/members/{account_id}"
                ))
                .await?;
            crate::output::print_success("Member removed");
        }
        BusinessCommand::HandleInvitation { token, action } => {
            let biz_id = client.require_business_id()?;
            let body = json!({ "token": token, "action": action });
            let result = client
                .put(&format!("/v1/businesses/{biz_id}/invitation"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::TestWebhook { data } => {
            let biz_id = client.require_business_id()?;
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/webhooks/test"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::Refund { data } => {
            let biz_id = client.require_business_id()?;
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/refund"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::OauthConnect { data } => {
            let biz_id = client.require_business_id()?;
            let body = parse_data(data.as_deref())?;
            let result = client
                .post(&format!("/v1/businesses/{biz_id}/oauth/connect"), &body)
                .await?;
            crate::output::print_output(&result, format);
        }
        BusinessCommand::OauthDisconnect { provider } => {
            let biz_id = client.require_business_id()?;
            let body = json!({ "provider": provider });
            let result = client
                .post(
                    &format!("/v1/businesses/{biz_id}/oauth/disconnect"),
                    &body,
                )
                .await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
