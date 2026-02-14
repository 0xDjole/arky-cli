use std::fmt;

#[derive(Debug)]
pub enum CliError {
    Http(reqwest::Error),
    Api {
        status: u16,
        message: String,
        error: Option<String>,
        validation_errors: Vec<ValidationError>,
    },
    Config(String),
    InvalidInput(String),
    Io(std::io::Error),
    Json(serde_json::Error),
}

#[derive(Debug, serde::Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub error: String,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiErrorResponse {
    pub message: Option<String>,
    pub error: Option<String>,
    pub status_code: Option<u16>,
    #[serde(default)]
    pub validation_errors: Vec<ValidationError>,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::Http(e) => write!(f, "HTTP error: {e}"),
            CliError::Api {
                status,
                message,
                error,
                validation_errors,
            } => {
                write!(f, "API error ({status}): {message}")?;
                if let Some(err) = error {
                    write!(f, " [{err}]")?;
                }
                for ve in validation_errors {
                    write!(f, "\n  - {}: {}", ve.field, ve.error)?;
                }
                Ok(())
            }
            CliError::Config(msg) => write!(f, "Config error: {msg}"),
            CliError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            CliError::Io(e) => write!(f, "IO error: {e}"),
            CliError::Json(e) => write!(f, "JSON error: {e}"),
        }
    }
}

impl std::error::Error for CliError {}

impl From<reqwest::Error> for CliError {
    fn from(e: reqwest::Error) -> Self {
        CliError::Http(e)
    }
}

impl From<std::io::Error> for CliError {
    fn from(e: std::io::Error) -> Self {
        CliError::Io(e)
    }
}

impl From<serde_json::Error> for CliError {
    fn from(e: serde_json::Error) -> Self {
        CliError::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, CliError>;
