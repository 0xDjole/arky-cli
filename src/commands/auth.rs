use crate::client::ArkyClient;
use crate::config::Config;
use crate::error::Result;
use crate::output::{print_success, Format};
use clap::Subcommand;
use serde_json::json;

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Send a magic link code to an email address
    #[command(long_about = "Request a verification code via email (magic link).\n\n\
        This is step 1 of authentication. A 6-digit code is sent to the email.\n\
        Use `arky auth verify` with the code to complete login.\n\n\
        Example:\n\
        arky auth login user@example.com\n\n\
        Response: {\"success\": true}")]
    Login {
        /// Email address to send the code to
        email: String,
    },
    /// Verify a magic link code and save the token
    #[command(long_about = "Verify email with the code received, get auth token.\n\n\
        This is step 2 of authentication. On success, the access token is\n\
        automatically saved to ~/.arky/config.json for future requests.\n\n\
        Example:\n\
        arky auth verify user@example.com 123456\n\n\
        Response: {\"accessToken\": \"eyJ...\", \"refreshToken\": \"...\", \"accountId\": \"...\"}")]
    Verify {
        /// Email address
        email: String,
        /// Verification code received via email
        code: String,
    },
    /// Create an anonymous session (no email needed)
    #[command(long_about = "Create an anonymous session token.\n\n\
        Useful for public-facing operations that don't require a user account.\n\
        The token is saved to ~/.arky/config.json.\n\n\
        Example:\n\
        arky auth session\n\n\
        Response: {\"accessToken\": \"eyJ...\", \"accountId\": \"anon_...\"}")]
    Session,
    /// Show current account info
    #[command(long_about = "Display the account associated with the current token.\n\n\
        Requires a valid token (set via login/verify, session, or --token flag).\n\n\
        Example:\n\
        arky auth whoami\n\n\
        Response: {\"id\": \"acc_123\", \"email\": \"user@example.com\", \"name\": \"...\"}")]
    Whoami,
}

pub async fn handle(cmd: AuthCommand, client: &ArkyClient, format: &Format) -> Result<()> {
    match cmd {
        AuthCommand::Login { email } => {
            let result = client
                .post("/v1/auth/code", &json!({ "email": email }))
                .await?;
            print_success(&format!("Code sent to {email}"));
            crate::output::print_output(&result, format);
        }
        AuthCommand::Verify { email, code } => {
            let result = client
                .post("/v1/auth/verify", &json!({ "email": email, "code": code }))
                .await?;

            // Save token to config
            if let Some(token) = result.get("accessToken").and_then(|v| v.as_str()) {
                let mut cfg = Config::load_file();
                cfg.token = Some(token.to_string());
                cfg.save_file()?;
                print_success("Token saved to ~/.arky/config.json");
            }

            crate::output::print_output(&result, format);
        }
        AuthCommand::Session => {
            let result = client.post("/v1/auth/session", &json!({})).await?;

            if let Some(token) = result.get("accessToken").and_then(|v| v.as_str()) {
                let mut cfg = Config::load_file();
                cfg.token = Some(token.to_string());
                cfg.save_file()?;
                print_success("Session token saved to ~/.arky/config.json");
            }

            crate::output::print_output(&result, format);
        }
        AuthCommand::Whoami => {
            let result = client.get("/v1/accounts/me", &[]).await?;
            crate::output::print_output(&result, format);
        }
    }
    Ok(())
}
