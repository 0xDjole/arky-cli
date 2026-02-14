//! Integration tests that target a running local Arky server.
//!
//! These tests are #[ignore]d by default. Run them with:
//!   cargo test -- --ignored
//!
//! Prerequisites:
//!   1. Start the Arky server locally (default port 8000)
//!   2. Ensure dev.json config is loaded (has admin API token)
//!
//! Block format:
//!   Blocks are tagged enums: {"type":"text","id":"uuid","key":"title","properties":{},"value":"Hello"}
//!   All required fields: type, id, key, properties, value

use assert_cmd::Command;
use serde_json::Value;
use std::io::Write;

const BASE_URL: &str = "http://localhost:8000";
const API_TOKEN: &str = "arky_dev_admin_token_2025";
const BUSINESS_ID: &str = "0bbf0256-2fe9-4517-81ff-ebf8ebb2f373";

fn arky() -> Command {
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("arky").unwrap();
    cmd.env("ARKY_BASE_URL", BASE_URL)
        .env("ARKY_TOKEN", API_TOKEN)
        .env("ARKY_BUSINESS_ID", BUSINESS_ID);
    cmd
}

fn json_output(cmd: &mut Command) -> Value {
    let output = cmd.output().expect("failed to execute");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "Command failed.\nstdout: {}\nstderr: {}",
        stdout,
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(&stdout).unwrap_or_else(|e| {
        panic!("Failed to parse JSON: {e}\nOutput: {stdout}");
    })
}

fn uuid() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    format!(
        "{:08x}-{:04x}-4{:03x}-{:04x}-{:012x}",
        ts.as_secs() as u32,
        (ts.as_nanos() >> 16) as u16 & 0xFFFF,
        (ts.as_nanos() >> 32) as u16 & 0x0FFF,
        0x8000 | ((ts.as_nanos() >> 48) as u16 & 0x3FFF),
        ts.as_nanos() as u64 & 0xFFFF_FFFF_FFFF
    )
}

fn ts_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

fn has_list_items(val: &Value) -> bool {
    val.get("items").is_some() || val.get("data").is_some()
}

// ── Business CRUD ───────────────────────────────────────────

#[test]
#[ignore]
fn test_business_crud() {
    let key = format!("cli-test-biz-{}", ts_ms());

    // Create
    let create_data = serde_json::json!({
        "status": "active",
        "timezone": "UTC",
        "billingEmail": "test@cli.dev",
        "configs": {
            "languages": [{"id": "en"}],
            "markets": [{"id": "us", "currency": "usd", "taxMode": "exclusive"}],
            "zones": [],
            "locations": [],
            "buildHooks": [],
            "webhooks": [],
            "integrations": [],
            "shippingIds": [],
            "analyticsIds": [],
            "emails": {"billing": "test@cli.dev", "support": "test@cli.dev"}
        }
    });

    let val = json_output(
        arky().args(["business", "create", &key, "--data", &create_data.to_string()])
    );
    let biz_id = val["id"].as_str().expect("Business should have id");
    assert_eq!(val["key"].as_str().unwrap(), key);
    assert_eq!(val["status"].as_str().unwrap(), "active");

    // Get
    let val = json_output(
        &mut arky()
            .env("ARKY_BUSINESS_ID", biz_id)
            .args(["business", "get"])
    );
    assert_eq!(val["id"].as_str().unwrap(), biz_id);
    assert_eq!(val["key"].as_str().unwrap(), key);

    // List (should include our new business)
    let val = json_output(&mut arky().args(["business", "list", "--limit", "50"]));
    assert!(has_list_items(&val), "Should have items. Got: {val}");

    // Delete
    arky().args(["business", "delete", biz_id]).assert().success();
}

// ── Nodes (CMS) ─────────────────────────────────────────────

#[test]
#[ignore]
fn test_node_crud() {
    let key = format!("cli-test-node-{}", ts_ms());

    let create_data = serde_json::json!({
        "slug": {"en": &key},
        "writeAccess": "private",
        "audienceIds": [],
        "blocks": [
            {"type": "localized_text", "id": uuid(), "key": "title", "properties": {}, "value": {"en": "CLI Test Node"}},
            {"type": "markdown", "id": uuid(), "key": "body", "properties": {}, "value": {"en": "# Hello\nFrom integration test"}},
            {"type": "number", "id": uuid(), "key": "count", "properties": {}, "value": 42},
            {"type": "boolean", "id": uuid(), "key": "visible", "properties": {}, "value": true}
        ]
    });

    let val = json_output(
        arky().args(["node", "create", &key, "--data", &create_data.to_string()])
    );
    let node_id = val["id"].as_str().expect("Created node should have id");
    assert_eq!(val["key"].as_str().unwrap(), key);

    // Get
    let val = json_output(&mut arky().args(["node", "get", node_id]));
    assert_eq!(val["id"].as_str().unwrap(), node_id);
    let blocks = val["blocks"].as_array().expect("Should have blocks");
    assert!(blocks.len() >= 4, "Should have at least 4 blocks, got {}", blocks.len());
    let title_block = blocks.iter().find(|b| b["key"] == "title").expect("Should have title block");
    assert_eq!(title_block["type"], "localized_text");
    assert_eq!(title_block["value"]["en"], "CLI Test Node");

    // Update
    let update_data = serde_json::json!({
        "key": &key,
        "slug": {"en": &key},
        "status": "active",
        "writeAccess": "private",
        "audienceIds": [],
        "blocks": [
            {"type": "localized_text", "id": uuid(), "key": "title", "properties": {}, "value": {"en": "Updated Title"}},
            {"type": "markdown", "id": uuid(), "key": "body", "properties": {}, "value": {"en": "# Updated"}}
        ]
    });
    let val = json_output(
        arky().args(["node", "update", node_id, "--data", &update_data.to_string()])
    );
    let blocks = val["blocks"].as_array().expect("Updated should have blocks");
    let title = blocks.iter().find(|b| b["key"] == "title").expect("title");
    assert_eq!(title["value"]["en"], "Updated Title");

    // List
    let val = json_output(&mut arky().args(["node", "list", "--limit", "5"]));
    assert!(has_list_items(&val));

    // Delete
    arky().args(["node", "delete", node_id]).assert().success();
}

// ── Products ────────────────────────────────────────────────

#[test]
#[ignore]
fn test_product_crud() {
    let key = format!("cli-test-product-{}", ts_ms());

    let create_data = serde_json::json!({
        "slug": {"en": &key},
        "status": "active",
        "audienceIds": [],
        "networkIds": [],
        "filters": [],
        "blocks": [
            {"type": "localized_text", "id": uuid(), "key": "title", "properties": {}, "value": {"en": "Test Product"}},
            {"type": "markdown", "id": uuid(), "key": "description", "properties": {}, "value": {"en": "# Test\nA test product"}}
        ],
        "variants": [{
            "key": "default",
            "prices": [{"currency": "usd", "market": "us", "amount": 1999}],
            "inventory": [{"locationId": "default", "available": 100, "reserved": 0}],
            "attributes": []
        }]
    });

    let val = json_output(
        arky().args(["product", "create", &key, "--data", &create_data.to_string()])
    );
    let product_id = val["id"].as_str().expect("Product should have id");

    // Get
    let val = json_output(&mut arky().args(["product", "get", product_id]));
    assert_eq!(val["key"].as_str().unwrap(), key);
    let variants = val["variants"].as_array().expect("Should have variants");
    assert!(!variants.is_empty());

    // List
    let val = json_output(&mut arky().args(["product", "list", "--limit", "5"]));
    assert!(has_list_items(&val));

    // Delete
    arky().args(["product", "delete", product_id]).assert().success();
}

// ── Provider + Service ──────────────────────────────────────

#[test]
#[ignore]
fn test_provider_service_flow() {
    let ts = ts_ms();

    // Create provider — all required fields
    let provider_key = format!("cli-test-provider-{ts}");
    let provider_data = serde_json::json!({
        "slug": {"en": &provider_key},
        "status": "active",
        "networkIds": [],
        "audienceIds": [],
        "filters": [],
        "blocks": [
            {"type": "localized_text", "id": uuid(), "key": "name", "properties": {}, "value": {"en": "Test Provider"}},
            {"type": "markdown", "id": uuid(), "key": "bio", "properties": {}, "value": {"en": "Integration test provider"}}
        ],
        "concurrentLimit": 1
    });

    let val = json_output(
        arky().args(["provider", "create", &provider_key, "--data", &provider_data.to_string()])
    );
    let provider_id = val["id"].as_str().expect("Provider should have id");

    // Create service with this provider
    let service_key = format!("cli-test-service-{ts}");
    let service_data = serde_json::json!({
        "slug": {"en": &service_key},
        "status": "active",
        "networkIds": [],
        "audienceIds": [],
        "filters": [],
        "blocks": [
            {"type": "localized_text", "id": uuid(), "key": "title", "properties": {}, "value": {"en": "Test Service"}}
        ],
        "providers": [{
            "providerId": provider_id,
            "prices": [{"currency": "usd", "market": "us", "amount": 5000}],
            "durations": [{"duration": 3600000, "isPause": false}],
            "isApprovalRequired": false,
            "audienceIds": [],
            "workingTime": {
                "workingDays": [
                    {"day": "monday", "workingHours": [{"from": 32400000, "to": 61200000}]},
                    {"day": "tuesday", "workingHours": [{"from": 32400000, "to": 61200000}]},
                    {"day": "wednesday", "workingHours": [{"from": 32400000, "to": 61200000}]},
                    {"day": "thursday", "workingHours": [{"from": 32400000, "to": 61200000}]},
                    {"day": "friday", "workingHours": [{"from": 32400000, "to": 61200000}]}
                ],
                "outcastDates": [],
                "specificDates": []
            }
        }]
    });

    let val = json_output(
        arky().args(["service", "create", &service_key, "--data", &service_data.to_string()])
    );
    let service_id = val["id"].as_str().expect("Service should have id");

    // List services
    let val = json_output(&mut arky().args(["service", "list", "--limit", "5"]));
    assert!(has_list_items(&val));

    // Get provider
    let val = json_output(&mut arky().args(["provider", "get", provider_id]));
    assert_eq!(val["key"].as_str().unwrap(), provider_key);

    // Cleanup
    arky().args(["service", "delete", service_id]).assert().success();
    arky().args(["provider", "delete", provider_id]).assert().success();
}

// ── Workflow ────────────────────────────────────────────────

#[test]
#[ignore]
fn test_workflow_crud() {
    let key = format!("cli-test-workflow-{}", ts_ms());

    let create_data = serde_json::json!({
        "status": "draft",
        "nodes": {
            "trigger": {"type": "trigger"},
            "process": {
                "type": "transform",
                "code": "trigger",
                "edges": [{"node": "trigger", "output": "default"}]
            }
        }
    });

    let val = json_output(
        arky().args(["workflow", "create", &key, "--data", &create_data.to_string()])
    );
    let workflow_id = val["id"].as_str().expect("Workflow should have id");

    // Get
    let val = json_output(&mut arky().args(["workflow", "get", workflow_id]));
    assert_eq!(val["key"].as_str().unwrap(), key);
    assert!(val.get("nodes").is_some(), "Should have nodes");

    // List
    let val = json_output(&mut arky().args(["workflow", "list", "--limit", "5"]));
    assert!(has_list_items(&val));

    // List executions
    let val = json_output(
        &mut arky().args(["workflow", "executions", workflow_id, "--limit", "5"])
    );
    assert!(has_list_items(&val));

    // Delete
    arky().args(["workflow", "delete", workflow_id]).assert().success();
}

// ── Media Upload ────────────────────────────────────────────

#[test]
#[ignore]
fn test_media_upload_and_list() {
    // 1x1 PNG
    let png_data: Vec<u8> = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
        0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
        0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53,
        0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41,
        0x54, 0x08, 0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00,
        0x00, 0x00, 0x02, 0x00, 0x01, 0xE2, 0x21, 0xBC,
        0x33, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E,
        0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    let tmp_dir = tempfile::tempdir().unwrap();
    let file_path = tmp_dir.path().join("test-upload.png");
    let mut file = std::fs::File::create(&file_path).unwrap();
    file.write_all(&png_data).unwrap();
    drop(file);

    let val = json_output(
        arky().args(["media", "upload", file_path.to_str().unwrap()])
    );

    let items = val.as_array().expect("Upload should return array");
    assert!(!items.is_empty(), "Should have uploaded at least one file");

    let media = &items[0];
    let media_id = media["id"].as_str().expect("Media should have id");
    assert!(media.get("mimeType").is_some());
    assert!(media.get("resolutions").is_some());

    // List media
    let val = json_output(&mut arky().args(["media", "list", "--limit", "5"]));
    assert!(has_list_items(&val));

    // Delete uploaded media
    arky().args(["media", "delete", media_id]).assert().success();
}

// ── Audience ────────────────────────────────────────────────

#[test]
#[ignore]
fn test_audience_crud() {
    let key = format!("cli-test-audience-{}", ts_ms());

    let create_data = serde_json::json!({
        "prices": []
    });

    let val = json_output(
        arky().args(["audience", "create", &key, "--data", &create_data.to_string()])
    );
    let audience_id = val["id"].as_str().expect("Audience should have id");

    // Get
    let val = json_output(&mut arky().args(["audience", "get", audience_id]));
    assert_eq!(val["key"].as_str().unwrap(), key);

    // List
    let val = json_output(&mut arky().args(["audience", "list", "--limit", "5"]));
    assert!(has_list_items(&val));

    // Subscribers
    let val = json_output(
        &mut arky().args(["audience", "subscribers", audience_id])
    );
    assert!(has_list_items(&val));

    // Delete
    arky().args(["audience", "delete", audience_id]).assert().success();
}

// ── Database (KV) ───────────────────────────────────────────

#[test]
#[ignore]
fn test_db_put_scan_delete() {
    let key = format!("cli-test/{}", ts_ms());
    let value = serde_json::json!({"name": "test", "count": 42});

    // Put
    arky()
        .args(["db", "put", &key, "--value", &value.to_string()])
        .assert()
        .success();

    // Scan
    let val = json_output(&mut arky().args(["db", "scan", "cli-test/", "--limit", "10"]));
    let stdout = serde_json::to_string(&val).unwrap();
    assert!(stdout.contains("cli-test/"), "Scan should include our key prefix");

    // Delete
    arky().args(["db", "delete", &key]).assert().success();
}

// ── Promo Code CRUD ─────────────────────────────────────────

#[test]
#[ignore]
fn test_promo_code_crud() {
    let code = format!("CLITEST{}", ts_ms());

    // Create — ConditionValue is adjacently tagged: {"type":"count","value":50}
    let create_data = serde_json::json!({
        "code": &code,
        "discounts": [{"type": "items_percentage", "marketId": "us", "bps": 1500}],
        "conditions": [{"type": "max_uses", "value": {"type": "count", "value": 50}}]
    });
    let val = json_output(
        arky().args(["promo-code", "create", "--data", &create_data.to_string()])
    );
    let promo_id = val["id"].as_str().expect("Promo code should have id");
    assert_eq!(val["code"].as_str().unwrap(), code);

    // Get
    let val = json_output(&mut arky().args(["promo-code", "get", promo_id]));
    assert_eq!(val["id"].as_str().unwrap(), promo_id);
    assert_eq!(val["code"].as_str().unwrap(), code);

    // List
    let val = json_output(&mut arky().args(["promo-code", "list", "--limit", "5"]));
    assert!(has_list_items(&val));

    // Delete
    arky().args(["promo-code", "delete", promo_id]).assert().success();
}

// ── Order + Quote ───────────────────────────────────────────

#[test]
#[ignore]
fn test_order_quote_and_create() {
    // First, ensure business has a location for inventory
    let biz = json_output(&mut arky().args(["business", "get"]));
    let biz_id = biz["id"].as_str().expect("biz id");
    let biz_key = biz["key"].as_str().expect("biz key");
    let mut configs = biz["configs"].clone();
    let locations = configs["locations"].as_array().cloned().unwrap_or_default();
    let location_id = if locations.is_empty() {
        // Add a test location to the business
        let loc_id = uuid();
        configs["locations"] = serde_json::json!([{
            "id": &loc_id,
            "key": "test-warehouse",
            "address": {"name":"Test","street1":"1 Main","city":"NYC","state":"NY","postalCode":"10001","country":"US"},
            "isPickupLocation": false
        }]);
        let update_data = serde_json::json!({
            "key": biz_key,
            "status": "active",
            "configs": configs,
            "timezone": biz["timezone"].as_str().unwrap_or("UTC")
        });
        json_output(
            arky().args(["business", "update", biz_id, "--data", &update_data.to_string()])
        );
        loc_id
    } else {
        locations[0]["id"].as_str().unwrap().to_string()
    };

    // Create product with inventory at the location
    let key = format!("cli-test-order-product-{}", ts_ms());
    let product_data = serde_json::json!({
        "slug": {"en": &key},
        "status": "active",
        "audienceIds": [],
        "networkIds": [],
        "filters": [],
        "blocks": [
            {"type": "localized_text", "id": uuid(), "key": "title", "properties": {}, "value": {"en": "Order Test Product"}}
        ],
        "variants": [{
            "key": "default",
            "prices": [{"currency": "usd", "market": "us", "amount": 2500}],
            "inventory": [{"locationId": &location_id, "available": 100, "reserved": 0}],
            "attributes": []
        }]
    });
    let product = json_output(
        arky().args(["product", "create", &key, "--data", &product_data.to_string()])
    );
    let product_id = product["id"].as_str().expect("product id");
    let variant_id = product["variants"][0]["id"].as_str().expect("variant id");

    // Quote
    let quote_data = serde_json::json!({
        "market": "us",
        "items": [{"productId": product_id, "variantId": variant_id, "quantity": 2}],
        "blocks": []
    });
    let val = json_output(
        arky().args(["order", "quote", "--data", &quote_data.to_string()])
    );
    assert!(val.get("total").is_some() || val.get("items").is_some(),
        "Quote should have total or items. Got: {val}");

    // Create order
    let order_data = serde_json::json!({
        "market": "us",
        "items": [{"productId": product_id, "variantId": variant_id, "quantity": 1}],
        "blocks": []
    });
    let val = json_output(
        arky().args(["order", "create", "--data", &order_data.to_string()])
    );
    let order_id = val["id"].as_str().expect("order id");

    // Get order
    let val = json_output(&mut arky().args(["order", "get", order_id]));
    assert_eq!(val["id"].as_str().unwrap(), order_id);

    // List orders
    let val = json_output(&mut arky().args(["order", "list", "--limit", "5"]));
    assert!(has_list_items(&val));

    // Cleanup
    arky().args(["product", "delete", product_id]).assert().success();
}

// ── Error handling ──────────────────────────────────────────

#[test]
#[ignore]
fn test_get_nonexistent_node_returns_error() {
    arky()
        .args(["node", "get", "nonexistent-id-12345"])
        .assert()
        .failure();
}

#[test]
#[ignore]
fn test_invalid_json_data() {
    arky()
        .args(["node", "create", "test", "--data", "not-json"])
        .assert()
        .failure();
}
