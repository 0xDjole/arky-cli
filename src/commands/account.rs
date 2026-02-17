use crate::client::ArkyClient;
use crate::commands::parse_data;
use crate::error::Result;
use crate::output::Format;
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum AccountCommand {
    /// Search accounts
    #[command(long_about = "Search for accounts by query.\n\n\
        Examples:\n\
        arky account search --query \"john\"\n\
        arky account search --query \"user@example.com\" --limit 5\n\n\
        Response shape:\n\
        {\"data\": [{\"id\": \"...\", \"email\": \"...\", \"name\": \"...\"}], \"cursor\": \"...\"}")]
    Search {
        #[arg(long)]
        query: Option<String>,
        #[arg(long, default_value = "20")]
        limit: u32,
        #[arg(long)]
        cursor: Option<String>,
    },
    /// Update the current account
    #[command(long_about = "Update the current account profile.\n\n\
        Optional (--data JSON):\n\
          phoneNumbers  Array of phone number strings\n\
          addresses     Array of address objects\n\
          apiTokens     Array of API tokens (null to clear)\n\n\
        Example:\n\
        arky account update --data '{\"phoneNumbers\": [\"+1234567890\"]}'")]
    Update {
        #[arg(long, help = "JSON data: inline, @file, or - for stdin")]
        data: Option<String>,
    },
    /// Delete the current account
    #[command(long_about = "Permanently delete the current account.\n\n\
        WARNING: This is irreversible.\n\n\
        Example:\n\
        arky account delete")]
    Delete,
    /// Add a phone number to the account (sends verification code)
    #[command(name = "add-phone", long_about = "Add a phone number to the current account.\n\n\
        Sends a verification code to the phone number.\n\
        Use `arky account confirm-phone` to complete verification.\n\n\
        Required:\n\
          --phone   Phone number (e.g. \"+1234567890\")\n\n\
        Example:\n\
        arky account add-phone --phone \"+1234567890\"")]
    AddPhone {
        #[arg(long)]
        phone: String,
    },
    /// Confirm a phone number with verification code
    #[command(name = "confirm-phone", long_about = "Confirm a phone number with the code received.\n\n\
        Required:\n\
          --phone   Phone number being verified\n\
          --code    Verification code received via SMS\n\n\
        Example:\n\
        arky account confirm-phone --phone \"+1234567890\" --code 123456")]
    ConfirmPhone {
        #[arg(long)]
        phone: String,
        #[arg(long)]
        code: String,
    },
}

pub async fn handle(cmd: AccountCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    match cmd {
        AccountCommand::Search {
            query,
            limit,
            cursor,
        } => {
            let biz_id = client.require_business_id()?;
            let mut params: Vec<(&str, String)> = vec![
                ("limit", limit.to_string()),
                ("businessId", biz_id.to_string()),
            ];
            if let Some(ref q) = query {
                params.push(("query", q.clone()));
            }
            if let Some(ref c) = cursor {
                params.push(("cursor", c.clone()));
            }
            let params_ref: Vec<(&str, &str)> =
                params.iter().map(|(k, v)| (*k, v.as_str())).collect();
            let result = client.get("/v1/accounts/search", &params_ref).await?;
            crate::output::print_output(&result, format);
        }
        AccountCommand::Update { data } => {
            let body = parse_data(data.as_deref())?;
            let result = client.put("/v1/accounts", &body).await?;
            crate::output::print_output(&result, format);
        }
        AccountCommand::Delete => {
            let _ = client.delete("/v1/accounts").await?;
            crate::output::print_success("Account deleted");
        }
        AccountCommand::AddPhone { phone } => {
            let body = json!({ "phoneNumber": phone });
            let _ = client.post("/v1/accounts/phone-number", &body).await?;
            crate::output::print_success(&format!("Verification code sent to {phone}"));
        }
        AccountCommand::ConfirmPhone { phone, code } => {
            let body = json!({ "phoneNumber": phone, "code": code });
            let _ = client
                .post("/v1/accounts/phone-number/confirm", &body)
                .await?;
            crate::output::print_success("Phone number confirmed");
        }
    }
    Ok(())
}
