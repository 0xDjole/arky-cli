use colored::Colorize;
use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub enum Format {
    Json,
    Table,
    Plain,
}

impl Format {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "table" => Format::Table,
            "plain" => Format::Plain,
            _ => Format::Json,
        }
    }
}

pub fn print_output(value: &Value, format: &Format) {
    match format {
        Format::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(value).unwrap_or_else(|_| value.to_string())
            );
        }
        Format::Table => print_table(value),
        Format::Plain => print_plain(value),
    }
}

pub fn print_success(msg: &str) {
    eprintln!("{} {}", "OK".green().bold(), msg);
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "ERROR".red().bold(), msg);
}

fn print_table(value: &Value) {
    match value {
        Value::Array(items) => {
            if items.is_empty() {
                println!("(empty)");
                return;
            }

            // Collect keys from first item
            let keys: Vec<String> = if let Some(Value::Object(obj)) = items.first() {
                obj.keys()
                    .filter(|k| {
                        // Skip large nested objects in table view
                        if let Some(first) = items.first() {
                            !matches!(first.get(k.as_str()), Some(Value::Array(_)) | Some(Value::Object(_)))
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect()
            } else {
                // Not objects, just print values
                for item in items {
                    println!("{}", format_cell(item));
                }
                return;
            };

            if keys.is_empty() {
                println!("(no scalar fields)");
                return;
            }

            // Calculate column widths
            let mut widths: Vec<usize> = keys.iter().map(|k| k.len()).collect();
            for item in items {
                for (i, key) in keys.iter().enumerate() {
                    let cell = format_cell(item.get(key).unwrap_or(&Value::Null));
                    widths[i] = widths[i].max(cell.len().min(40));
                }
            }

            // Print header
            let header: Vec<String> = keys
                .iter()
                .enumerate()
                .map(|(i, k)| format!("{:width$}", k.to_uppercase(), width = widths[i]))
                .collect();
            println!("{}", header.join("  ").bold());
            let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
            println!("{}", sep.join("  ").dimmed());

            // Print rows
            for item in items {
                let row: Vec<String> = keys
                    .iter()
                    .enumerate()
                    .map(|(i, key)| {
                        let cell = format_cell(item.get(key).unwrap_or(&Value::Null));
                        let truncated = if cell.len() > 40 {
                            format!("{}...", &cell[..37])
                        } else {
                            cell
                        };
                        format!("{:width$}", truncated, width = widths[i])
                    })
                    .collect();
                println!("{}", row.join("  "));
            }
        }
        Value::Object(obj) => {
            let max_key_len = obj.keys().map(|k| k.len()).max().unwrap_or(0);
            for (key, val) in obj {
                let display = format_cell(val);
                let truncated = if display.len() > 80 {
                    format!("{}...", &display[..77])
                } else {
                    display
                };
                println!(
                    "{:>width$}  {}",
                    key.bold(),
                    truncated,
                    width = max_key_len
                );
            }
        }
        other => println!("{}", format_cell(other)),
    }
}

fn print_plain(value: &Value) {
    match value {
        Value::String(s) => println!("{s}"),
        Value::Number(n) => println!("{n}"),
        Value::Bool(b) => println!("{b}"),
        Value::Null => println!("null"),
        Value::Array(items) => {
            for item in items {
                print_plain(item);
            }
        }
        Value::Object(obj) => {
            for (key, val) in obj {
                println!("{}={}", key, format_cell(val));
            }
        }
    }
}

fn format_cell(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "-".to_string(),
        Value::Array(a) => format!("[{} items]", a.len()),
        Value::Object(_) => "{...}".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_str() {
        assert_eq!(Format::from_str("json"), Format::Json);
        assert_eq!(Format::from_str("table"), Format::Table);
        assert_eq!(Format::from_str("plain"), Format::Plain);
        assert_eq!(Format::from_str("JSON"), Format::Json);
        assert_eq!(Format::from_str("unknown"), Format::Json);
    }

    #[test]
    fn test_format_cell() {
        assert_eq!(format_cell(&Value::String("hello".into())), "hello");
        assert_eq!(format_cell(&Value::Number(42.into())), "42");
        assert_eq!(format_cell(&Value::Bool(true)), "true");
        assert_eq!(format_cell(&Value::Null), "-");
        assert_eq!(
            format_cell(&serde_json::json!([1, 2, 3])),
            "[3 items]"
        );
        assert_eq!(format_cell(&serde_json::json!({"a": 1})), "{...}");
    }
}
