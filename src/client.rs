use crate::error::{ApiErrorResponse, CliError, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::multipart;
use serde_json::Value;

pub struct ArkyClient {
    http: reqwest::Client,
    pub base_url: String,
    pub business_id: Option<String>,
    token: Option<String>,
}

impl ArkyClient {
    pub fn new(base_url: String, business_id: Option<String>, token: Option<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            business_id,
            token,
        }
    }

    pub fn require_business_id(&self) -> Result<&str> {
        self.business_id.as_deref().ok_or_else(|| {
            CliError::Config("business_id required".into())
        })
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        if let Some(ref token) = self.token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {token}")) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        headers
    }

    fn auth_headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("Accept", HeaderValue::from_static("application/json"));
        if let Some(ref token) = self.token {
            if let Ok(val) = HeaderValue::from_str(&format!("Bearer {token}")) {
                headers.insert(AUTHORIZATION, val);
            }
        }
        headers
    }

    pub async fn get(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .headers(self.headers())
            .query(params)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn post(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .headers(self.headers())
            .json(body)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn put(&self, path: &str, body: &Value) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .put(&url)
            .headers(self.headers())
            .json(body)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn delete(&self, path: &str) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .delete(&url)
            .headers(self.headers())
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn delete_with_params(&self, path: &str, params: &[(&str, &str)]) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .delete(&url)
            .headers(self.headers())
            .query(params)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    pub async fn upload(&self, path: &str, files: Vec<(String, Vec<u8>, String)>) -> Result<Value> {
        let url = format!("{}{}", self.base_url, path);
        let mut form = multipart::Form::new();

        for (i, (filename, data, mime)) in files.into_iter().enumerate() {
            let part = multipart::Part::bytes(data)
                .file_name(filename)
                .mime_str(&mime)
                .map_err(|e| CliError::InvalidInput(format!("Invalid MIME type: {e}")))?;
            form = form.part(format!("files[{i}]"), part);
        }

        let resp = self
            .http
            .post(&url)
            .headers(self.auth_headers())
            .multipart(form)
            .send()
            .await?;
        self.handle_response(resp).await
    }

    async fn handle_response(&self, resp: reqwest::Response) -> Result<Value> {
        let status = resp.status().as_u16();

        if status == 204 {
            return Ok(Value::Null);
        }

        let body = resp.text().await?;

        if status >= 400 {
            let api_err: ApiErrorResponse =
                serde_json::from_str(&body).unwrap_or(ApiErrorResponse {
                    message: Some(body.clone()),
                    error: None,
                    status_code: Some(status),
                    validation_errors: vec![],
                });

            return Err(CliError::Api {
                status,
                message: api_err.message.unwrap_or_else(|| "Request failed".into()),
                error: api_err.error,
                validation_errors: api_err.validation_errors,
            });
        }

        if body.is_empty() {
            return Ok(Value::Null);
        }

        serde_json::from_str(&body).map_err(CliError::from)
    }
}
