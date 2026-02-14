use assert_cmd::Command;
use predicates::prelude::*;

fn arky() -> Command {
    Command::cargo_bin("arky").unwrap()
}

// ── Help & Version ──────────────────────────────────────────

#[test]
fn test_help() {
    arky()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Arky CLI"))
        .stdout(predicate::str::contains("auth"))
        .stdout(predicate::str::contains("node"))
        .stdout(predicate::str::contains("product"))
        .stdout(predicate::str::contains("workflow"))
        .stdout(predicate::str::contains("booking"))
        .stdout(predicate::str::contains("service"))
        .stdout(predicate::str::contains("provider"))
        .stdout(predicate::str::contains("Block system"));
}

#[test]
fn test_version() {
    arky()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("arky 0.1.0"));
}

// ── Subcommand Help ─────────────────────────────────────────

#[test]
fn test_auth_help() {
    arky()
        .args(["auth", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("login"))
        .stdout(predicate::str::contains("verify"))
        .stdout(predicate::str::contains("session"))
        .stdout(predicate::str::contains("whoami"));
}

#[test]
fn test_node_help() {
    arky()
        .args(["node", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("children"));
}

#[test]
fn test_node_create_help_shows_blocks() {
    arky()
        .args(["node", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("blocks"))
        .stdout(predicate::str::contains("localized_text"))
        .stdout(predicate::str::contains("relationship_media"))
        .stdout(predicate::str::contains("@content.json"));
}

#[test]
fn test_node_update_help_shows_block_types() {
    arky()
        .args(["node", "update", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("text"))
        .stdout(predicate::str::contains("markdown"))
        .stdout(predicate::str::contains("geo_location"));
}

#[test]
fn test_workflow_help() {
    arky()
        .args(["workflow", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("trigger"))
        .stdout(predicate::str::contains("executions"))
        .stdout(predicate::str::contains("execution"));
}

#[test]
fn test_workflow_create_help_shows_node_types() {
    arky()
        .args(["workflow", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("trigger"))
        .stdout(predicate::str::contains("http"))
        .stdout(predicate::str::contains("switch"))
        .stdout(predicate::str::contains("transform"))
        .stdout(predicate::str::contains("loop"));
}

#[test]
fn test_product_help() {
    arky()
        .args(["product", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_product_create_help_shows_blocks() {
    arky()
        .args(["product", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("blocks"))
        .stdout(predicate::str::contains("variants"));
}

#[test]
fn test_service_create_help_shows_providers() {
    arky()
        .args(["service", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("providers"))
        .stdout(predicate::str::contains("workingTime"));
}

#[test]
fn test_provider_create_help_shows_blocks() {
    arky()
        .args(["provider", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("blocks"))
        .stdout(predicate::str::contains("concurrentLimit"));
}

#[test]
fn test_booking_help() {
    arky()
        .args(["booking", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("checkout"))
        .stdout(predicate::str::contains("quote"));
}

#[test]
fn test_db_help() {
    arky()
        .args(["db", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("scan"))
        .stdout(predicate::str::contains("put"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("run-script"));
}

#[test]
fn test_config_help() {
    arky()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("set"))
        .stdout(predicate::str::contains("path"));
}

#[test]
fn test_media_help() {
    arky()
        .args(["media", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("upload"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_media_upload_help() {
    arky()
        .args(["media", "upload", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("multipart"))
        .stdout(predicate::str::contains("50MB"))
        .stdout(predicate::str::contains("relationship_media"));
}

#[test]
fn test_shipping_help() {
    arky()
        .args(["shipping", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("rates"))
        .stdout(predicate::str::contains("ship"));
}

#[test]
fn test_promo_code_help() {
    arky()
        .args(["promo-code", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("delete"));
}

#[test]
fn test_promo_code_create_help_shows_discount_types() {
    arky()
        .args(["promo-code", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("items_percentage"))
        .stdout(predicate::str::contains("items_fixed"))
        .stdout(predicate::str::contains("shipping_percentage"))
        .stdout(predicate::str::contains("basis points"))
        .stdout(predicate::str::contains("max_uses"));
}

#[test]
fn test_service_create_help_shows_working_time_details() {
    arky()
        .args(["service", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("32400000"))
        .stdout(predicate::str::contains("61200000"))
        .stdout(predicate::str::contains("1800000"))
        .stdout(predicate::str::contains("outcastDates"))
        .stdout(predicate::str::contains("specificDates"));
}

#[test]
fn test_workflow_create_help_shows_expression_docs() {
    arky()
        .args(["workflow", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("boa"))
        .stdout(predicate::str::contains("back-edges"))
        .stdout(predicate::str::contains("switch"))
        .stdout(predicate::str::contains("transform"))
        .stdout(predicate::str::contains("loop"));
}

#[test]
fn test_top_level_help_shows_setup() {
    arky()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("ARKY_BASE_URL"))
        .stdout(predicate::str::contains("ARKY_TOKEN"))
        .stdout(predicate::str::contains("relationship_media"))
        .stdout(predicate::str::contains("localized_text"));
}

#[test]
fn test_audience_help() {
    arky()
        .args(["audience", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("get"))
        .stdout(predicate::str::contains("subscribers"));
}

#[test]
fn test_event_help() {
    arky()
        .args(["event", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));
}

// ── Config ──────────────────────────────────────────────────

#[test]
fn test_config_path() {
    arky()
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains(".arky/config.json"));
}

#[test]
fn test_config_show() {
    arky()
        .args(["config", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("base_url"));
}

// ── Global Flags ────────────────────────────────────────────

#[test]
fn test_format_flag() {
    arky()
        .args(["--format", "table", "config", "show"])
        .assert()
        .success();
}

#[test]
fn test_no_command_shows_help() {
    arky()
        .assert()
        .failure()
        .stderr(predicate::str::contains("Usage"));
}

// ── Error Cases ─────────────────────────────────────────────

#[test]
fn test_unknown_command() {
    arky()
        .arg("foobar")
        .assert()
        .failure();
}

#[test]
fn test_node_list_requires_business_id() {
    // Without business_id set, should fail with config error
    arky()
        .env_remove("ARKY_BUSINESS_ID")
        .args(["--business-id", "", "node", "list"])
        .assert()
        .failure();
}
