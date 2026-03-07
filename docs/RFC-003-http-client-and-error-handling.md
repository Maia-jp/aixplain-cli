# RFC-003: HTTP Client & Error Handling

**Status:** Draft  
**Authors:** Engineering Team  
**Created:** 2026-03-07  
**Dependencies:** RFC-001, RFC-002  

---

## Summary

Define the HTTP client layer, retry strategy, error type hierarchy, async operation polling, and streaming support for the aiXplain CLI. This is the transport foundation that all API operations build upon.

## Motivation

The aiXplain platform API has specific patterns: authentication headers, camelCase JSON, async operations with polling, SSE streaming, and S3 presigned URL uploads. A well-designed client layer encapsulates these patterns once and exposes a clean interface to the API operations layer.

## Client Architecture

```
┌──────────────────────────────────────────────────┐
│                  api/ layer                       │
│    (agents.rs, models.rs, tools.rs, ...)         │
├──────────────────────────────────────────────────┤
│                client/ layer                      │
│  ┌────────────┐ ┌──────────┐ ┌────────────────┐ │
│  │ AixClient  │ │  retry   │ │   polling      │ │
│  │            │ │          │ │                │ │
│  │ get()      │ │ policy   │ │ poll_until()   │ │
│  │ post()     │ │ backoff  │ │ exp. backoff   │ │
│  │ put()      │ │ jitter   │ │ timeout        │ │
│  │ delete()   │ │          │ │                │ │
│  │ stream()   │ │          │ │                │ │
│  │ upload()   │ │          │ │                │ │
│  └────────────┘ └──────────┘ └────────────────┘ │
├──────────────────────────────────────────────────┤
│              reqwest + tokio                      │
└──────────────────────────────────────────────────┘
```

## AixClient

The central HTTP client, constructed once at startup and shared across all API operations.

```rust
use reqwest::{Client, Response, StatusCode};
use url::Url;

pub struct AixClient {
    http: Client,
    backend_url: Url,
    models_run_url: Url,
    auth_header: (String, String),
    upload_auth_header: (String, String),
    retry_policy: RetryPolicy,
}

impl AixClient {
    pub fn new(config: &ResolvedConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(5)
            .user_agent(format!("aixplain-cli/{}", env!("CARGO_PKG_VERSION")))
            .build()?;

        let (header_name, header_value) = config.auth_header();
        let (upload_header_name, upload_header_value) = config.upload_auth_header();

        Ok(Self {
            http,
            backend_url: config.backend_url.clone(),
            models_run_url: config.models_run_url.clone(),
            auth_header: (header_name.to_string(), header_value.to_string()),
            upload_auth_header: (upload_header_name.to_string(), upload_header_value),
            retry_policy: RetryPolicy::default(),
        })
    }

    // Core request method — all other methods delegate here
    async fn request<T: DeserializeOwned>(
        &self,
        method: Method,
        url: Url,
        body: Option<&impl Serialize>,
        auth: AuthMode,
    ) -> Result<T, AixError>;

    // Convenience methods
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, AixError>;
    pub async fn post<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> Result<T, AixError>;
    pub async fn put<T: DeserializeOwned>(&self, path: &str, body: &impl Serialize) -> Result<T, AixError>;
    pub async fn delete(&self, path: &str) -> Result<(), AixError>;

    // Streaming response (for SSE model streaming)
    pub async fn stream(&self, url: &Url, body: &impl Serialize) -> Result<EventStream, AixError>;

    // Form data POST (for presigned URL requests — uses Authorization: token header)
    pub async fn post_form<T: DeserializeOwned>(&self, path: &str, form: reqwest::multipart::Form, auth: AuthMode) -> Result<T, AixError>;

    // File upload to S3 presigned URL (PUT with raw bytes)
    pub async fn upload_to_s3(&self, presigned_url: &str, data: Vec<u8>, content_type: &str) -> Result<(), AixError>;

    // Raw response (for polling URLs that return full response)
    pub async fn get_raw(&self, url: &str) -> Result<Response, AixError>;

    // POST to absolute URL (for MODELS_RUN_URL which is a full URL, not relative path)
    pub async fn post_url<T: DeserializeOwned>(&self, url: &Url, body: &impl Serialize) -> Result<T, AixError>;
}

enum AuthMode {
    Standard,
    Upload,
    None,
}
```

### URL Construction

Backend API paths are resolved against the base URL:

```rust
impl AixClient {
    fn backend_url(&self, path: &str) -> Url {
        self.backend_url.join(path)
            .unwrap_or_else(|_| panic!("Invalid API path: {path}"))
    }

    fn models_url(&self, model_id: &str) -> Url {
        let base = self.models_run_url.as_str().trim_end_matches('/');
        Url::parse(&format!("{base}/{model_id}"))
            .unwrap_or_else(|_| panic!("Invalid model URL for: {model_id}"))
    }
}
```

## Error Hierarchy

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AixError {
    #[error("API error ({status}): {message}")]
    Api {
        status: StatusCode,
        message: String,
        error_code: Option<String>,
        response_body: Option<String>,
    },

    #[error("Authentication failed: {0}")]
    Auth(String),

    #[error("Resource not found: {resource_type} '{id}'")]
    NotFound {
        resource_type: String,
        id: String,
    },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Operation timed out after {elapsed:?}")]
    Timeout {
        elapsed: Duration,
        operation: String,
    },

    #[error("Operation failed: {message}")]
    OperationFailed {
        message: String,
        supplier_error: Option<String>,
    },

    #[error("File upload failed: {0}")]
    Upload(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

impl AixError {
    /// User-friendly error message for CLI display
    pub fn user_message(&self) -> String {
        match self {
            Self::Auth(_) => format!("{self}\n\nRun `aix auth login` to configure credentials."),
            Self::NotFound { resource_type, id } =>
                format!("Could not find {resource_type} with ID '{id}'."),
            Self::Timeout { operation, .. } =>
                format!("{self}\n\nThe {operation} is taking longer than expected. Try again or increase timeout."),
            _ => self.to_string(),
        }
    }

    /// Whether this error should be retried
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Api { status, .. } => matches!(
                status.as_u16(),
                500 | 502 | 503 | 504 | 429
            ),
            Self::Http(e) => e.is_timeout() || e.is_connect(),
            _ => false,
        }
    }
}
```

### API Error Parsing

The aiXplain API returns errors in this shape:

```json
{
  "message": "Agent not found",
  "statusCode": 404,
  "error": "NOT_FOUND"
}
```

The client parses this and maps to specific `AixError` variants:

```rust
async fn parse_error_response(response: Response) -> AixError {
    let status = response.status();
    let body = response.text().await.unwrap_or_default();

    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        return AixError::Auth(format!("Request denied ({status})"));
    }

    let (message, error_code) = match serde_json::from_str::<ApiErrorBody>(&body) {
        Ok(err) => (err.message, err.error),
        Err(_) => (body.clone(), None),
    };

    if status == StatusCode::NOT_FOUND {
        return AixError::NotFound {
            resource_type: "resource".to_string(),
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiErrorBody {
    message: String,
    status_code: Option<u16>,
    error: Option<String>,
}
```

## Retry Policy

```rust
pub struct RetryPolicy {
    pub max_retries: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub backoff_factor: f64,
    pub retryable_statuses: Vec<u16>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 5,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(30),
            backoff_factor: 2.0,
            retryable_statuses: vec![500, 502, 503, 504, 429],
        }
    }
}
```

**SDK comparison:** The Python SDK uses `total=5, backoff_factor=0.1, status_forcelist=[500,502,503,504]`. The CLI matches the retry count (`5`), uses `initial_backoff=100ms` to align with the SDK's `0.1` factor, and adds `429` (rate limiting) since the CLI is more likely to hit rate limits in interactive use.

```
```

### Retry Loop

```rust
async fn request_with_retry<T: DeserializeOwned>(
    &self,
    method: Method,
    url: Url,
    body: Option<&impl Serialize>,
    auth: AuthMode,
) -> Result<T, AixError> {
    let mut last_error = None;
    let mut backoff = self.retry_policy.initial_backoff;

    for attempt in 0..=self.retry_policy.max_retries {
        match self.request_once(method.clone(), url.clone(), body, &auth).await {
            Ok(value) => return Ok(value),
            Err(e) if e.is_retryable() && attempt < self.retry_policy.max_retries => {
                let jitter = rand_jitter(backoff);
                tokio::time::sleep(jitter).await;
                backoff = (backoff.mul_f64(self.retry_policy.backoff_factor))
                    .min(self.retry_policy.max_backoff);
                last_error = Some(e);
            }
            Err(e) => return Err(e),
        }
    }

    Err(last_error.unwrap())
}
```

### Rate Limit Handling (429)

When a 429 is received:
1. Check for `Retry-After` header
2. If present, sleep for that duration
3. If absent, use exponential backoff
4. Always count against retry budget

## Polling for Async Operations

Many aiXplain operations (agent runs, model runs) return a polling URL rather than an immediate result. The client provides a generic polling mechanism.

```rust
pub struct PollConfig {
    pub initial_interval: Duration,
    pub min_interval: Duration,
    pub max_interval: Duration,
    pub backoff_factor: f64,
    pub timeout: Duration,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            initial_interval: Duration::from_millis(500),
            min_interval: Duration::from_millis(200),
            max_interval: Duration::from_secs(60),
            backoff_factor: 1.1,
            timeout: Duration::from_secs(300),
        }
    }
}

pub async fn poll_until_complete<T, F>(
    client: &AixClient,
    poll_url: &str,
    config: &PollConfig,
    mut on_poll: F,
) -> Result<T, AixError>
where
    T: DeserializeOwned,
    F: FnMut(&PollResponse),
{
    let start = Instant::now();
    let mut interval = config.initial_interval;

    loop {
        if start.elapsed() > config.timeout {
            return Err(AixError::Timeout {
                elapsed: start.elapsed(),
                operation: "poll".to_string(),
            });
        }

        let response: PollResponse = client.get_raw(poll_url).await?.json().await?;
        on_poll(&response);

        if response.status == "FAILED" {
            let message = response.supplier_error
                .or(response.error_message)
                .unwrap_or_else(|| "Operation failed".to_string());
            return Err(AixError::OperationFailed {
                message,
                supplier_error: None,
            });
        }

        if response.completed {
            if let Some(error) = response.error_message {
                return Err(AixError::OperationFailed {
                    message: error,
                    supplier_error: response.supplier_error,
                });
            }
            return Ok(serde_json::from_value(response.data.unwrap_or_default())?);
        }

        tokio::time::sleep(interval).await;
        interval = (interval.mul_f64(config.backoff_factor))
            .max(config.min_interval)
            .min(config.max_interval);
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollResponse {
    pub status: String,
    pub completed: bool,
    pub error_message: Option<String>,
    pub supplier_error: Option<String>,
    pub data: Option<serde_json::Value>,
    pub result: Option<serde_json::Value>,
    pub url: Option<String>,
    pub session_id: Option<String>,
    pub used_credits: Option<f64>,
    pub run_time: Option<f64>,
    pub request_id: Option<String>,
}
```

### Poll Progress Callback

The `on_poll` callback enables the CLI/TUI to show progress:

```rust
// CLI usage: show spinner
poll_until_complete(&client, &url, &config, |resp| {
    spinner.set_message(format!("Status: {}...", resp.status));
}).await?;

// TUI usage: update progress state
poll_until_complete(&client, &url, &config, |resp| {
    tx.send(Message::AgentProgress(resp.clone())).ok();
}).await?;
```

## Streaming (SSE)

For model streaming responses:

```rust
pub struct EventStream {
    inner: reqwest::Response,
    buffer: String,
}

impl EventStream {
    pub async fn next_event(&mut self) -> Option<Result<StreamEvent, AixError>> {
        loop {
            let chunk = self.inner.chunk().await;
            match chunk {
                Ok(Some(bytes)) => {
                    self.buffer.push_str(&String::from_utf8_lossy(&bytes));
                    if let Some(event) = self.parse_next_sse() {
                        return Some(Ok(event));
                    }
                }
                Ok(None) => return None,
                Err(e) => return Some(Err(e.into())),
            }
        }
    }

    fn parse_next_sse(&mut self) -> Option<StreamEvent> {
        // Parse SSE format: "data: {json}\n\n"
        // Terminate on "data: [DONE]"
    }
}

#[derive(Debug, Deserialize)]
pub struct StreamEvent {
    pub data: String,
    pub completed: bool,
}
```

## Request/Response Logging

In debug mode (`AIXPLAIN_DEBUG=1` or `--verbose`), the client logs:

```
→ POST https://platform-api.aixplain.com/v2/agents/paginate
  Headers: x-api-key: 581b...ac52
  Body: {"q":"","pageNumber":0,"pageSize":20}
← 200 OK (143ms)
  Body: {"results":[...],"total":5,"pageTotal":1}
```

Keys are always redacted. Request/response bodies are truncated at 1KB in logs.

## Connection Pooling

The `reqwest::Client` is constructed once and reused (HTTP/2 multiplexing, keep-alive). Configuration:

- `pool_max_idle_per_host(5)` — Up to 5 idle connections per host
- `timeout(30s)` — Overall request timeout
- `connect_timeout(10s)` — TCP connection timeout

## Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use wiremock::{MockServer, Mock, ResponseTemplate};
    use wiremock::matchers::{method, path, header};

    #[tokio::test]
    async fn test_retry_on_503() {
        let mock = MockServer::start().await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(503))
            .up_to_n_times(2)
            .mount(&mock).await;
        Mock::given(method("GET"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({"ok": true})))
            .mount(&mock).await;

        let client = test_client(&mock.uri());
        let result: serde_json::Value = client.get("test").await.unwrap();
        assert_eq!(result["ok"], true);
    }

    #[tokio::test]
    async fn test_auth_error_mapping() { /* ... */ }

    #[tokio::test]
    async fn test_poll_with_timeout() { /* ... */ }

    #[tokio::test]
    async fn test_sse_parsing() { /* ... */ }
}
```

## Acceptance Criteria

- [ ] `AixClient` sends correct auth headers based on key type
- [ ] Retries on 500/502/503/504 with exponential backoff + jitter
- [ ] Handles 429 with `Retry-After` header
- [ ] Parses API error responses into typed `AixError` variants
- [ ] `poll_until_complete` respects timeout and calls progress callback
- [ ] SSE stream parsing handles chunked data correctly
- [ ] File upload works with presigned S3 URLs
- [ ] Debug logging shows redacted request/response details
- [ ] All methods handle network errors gracefully
