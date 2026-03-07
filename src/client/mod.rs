pub mod error;
pub mod polling;

pub use error::AixError;

use crate::config::ResolvedConfig;
use reqwest::{Client, Method, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::time::Duration;
use url::Url;

pub struct AixClient {
    http: Client,
    pub backend_url: Url,
    pub models_run_url: Url,
    auth_header_name: String,
    auth_header_value: String,
    upload_auth_value: String,
}

impl AixClient {
    pub fn new(config: &ResolvedConfig) -> Result<Self, AixError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(5)
            .user_agent(format!("aixplain-cli/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(AixError::Http)?;

        Ok(Self {
            http,
            backend_url: config.backend_url.clone(),
            models_run_url: config.models_run_url.clone(),
            auth_header_name: config.auth_header_name().to_string(),
            auth_header_value: config.api_key.clone(),
            upload_auth_value: config.upload_auth_value(),
        })
    }

    fn backend_path(&self, path: &str) -> String {
        let base = self.backend_url.as_str().trim_end_matches('/');
        format!("{base}/{path}")
    }

    pub fn models_url(&self, id: &str) -> String {
        let base = self.models_run_url.as_str().trim_end_matches('/');
        format!("{base}/{id}")
    }

    async fn request_raw(
        &self,
        method: Method,
        url: &str,
        body: Option<&serde_json::Value>,
    ) -> Result<Response, AixError> {
        let mut req = self
            .http
            .request(method, url)
            .header(&self.auth_header_name, &self.auth_header_value);

        if let Some(b) = body {
            req = req.json(b);
        }

        let resp = req.send().await.map_err(AixError::Http)?;

        if resp.status().is_success() {
            Ok(resp)
        } else {
            Err(Self::parse_error(resp).await)
        }
    }

    async fn parse_error(resp: Response) -> AixError {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();

        if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
            return AixError::Auth(format!("Request denied ({status})"));
        }

        let (message, error_code) = match serde_json::from_str::<serde_json::Value>(&body) {
            Ok(v) => (
                v.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or(&body)
                    .to_string(),
                v.get("error").and_then(|e| e.as_str()).map(String::from),
            ),
            Err(_) => (body.clone(), None),
        };

        if status == reqwest::StatusCode::NOT_FOUND {
            return AixError::NotFound {
                resource_type: "resource".into(),
                id: String::new(),
            };
        }

        AixError::Api {
            status,
            message,
            error_code,
            response_body: Some(body),
        }
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, AixError> {
        let url = self.backend_path(path);
        let resp = self.request_raw(Method::GET, &url, None).await?;
        resp.json().await.map_err(AixError::Http)
    }

    pub async fn post<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, AixError> {
        let url = self.backend_path(path);
        let json_body = serde_json::to_value(body)?;
        let resp = self
            .request_raw(Method::POST, &url, Some(&json_body))
            .await?;
        resp.json().await.map_err(AixError::Http)
    }

    pub async fn post_url<T: DeserializeOwned>(
        &self,
        url: &str,
        body: &impl Serialize,
    ) -> Result<T, AixError> {
        let json_body = serde_json::to_value(body)?;
        let resp = self
            .request_raw(Method::POST, url, Some(&json_body))
            .await?;
        resp.json().await.map_err(AixError::Http)
    }

    pub async fn put<T: DeserializeOwned>(
        &self,
        path: &str,
        body: &impl Serialize,
    ) -> Result<T, AixError> {
        let url = self.backend_path(path);
        let json_body = serde_json::to_value(body)?;
        let resp = self
            .request_raw(Method::PUT, &url, Some(&json_body))
            .await?;
        resp.json().await.map_err(AixError::Http)
    }

    pub async fn delete(&self, path: &str) -> Result<(), AixError> {
        let url = self.backend_path(path);
        self.request_raw(Method::DELETE, &url, None).await?;
        Ok(())
    }

    pub async fn get_url_raw(&self, url: &str) -> Result<Response, AixError> {
        self.request_raw(Method::GET, url, None).await
    }

    pub async fn post_form_upload<T: DeserializeOwned>(
        &self,
        path: &str,
        form: reqwest::multipart::Form,
    ) -> Result<T, AixError> {
        let url = self.backend_path(path);
        let resp = self
            .http
            .post(&url)
            .header("Authorization", &self.upload_auth_value)
            .multipart(form)
            .send()
            .await
            .map_err(AixError::Http)?;

        if resp.status().is_success() {
            resp.json().await.map_err(AixError::Http)
        } else {
            Err(Self::parse_error(resp).await)
        }
    }

    pub async fn upload_to_s3(
        &self,
        presigned_url: &str,
        data: Vec<u8>,
        content_type: &str,
    ) -> Result<(), AixError> {
        let resp = self
            .http
            .put(presigned_url)
            .header("Content-Type", content_type)
            .body(data)
            .send()
            .await
            .map_err(AixError::Http)?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(AixError::Upload(format!(
                "S3 upload failed with status {}",
                resp.status()
            )))
        }
    }
}
