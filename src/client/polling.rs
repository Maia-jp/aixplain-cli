use super::AixClient;
use super::AixError;
use std::time::{Duration, Instant};

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

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollResponse {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub supplier_error: Option<String>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub used_credits: Option<f64>,
    #[serde(default)]
    pub run_time: Option<f64>,
    #[serde(default)]
    pub request_id: Option<String>,
}

pub async fn poll_until_complete(
    client: &AixClient,
    poll_url: &str,
    config: &PollConfig,
    on_poll: Option<&(dyn Fn(&PollResponse) + Send + Sync)>,
) -> Result<PollResponse, AixError> {
    let start = Instant::now();
    let mut interval = config.initial_interval;

    loop {
        if start.elapsed() > config.timeout {
            return Err(AixError::Timeout {
                elapsed: start.elapsed(),
                operation: "poll".to_string(),
            });
        }

        let resp = client.get_url_raw(poll_url).await?;
        let poll: PollResponse = resp.json().await.map_err(AixError::Http)?;

        if let Some(cb) = on_poll {
            cb(&poll);
        }

        if poll.status == "FAILED" {
            let message = poll
                .supplier_error
                .clone()
                .or(poll.error_message.clone())
                .unwrap_or_else(|| "Operation failed".to_string());
            return Err(AixError::OperationFailed {
                message,
                supplier_error: poll.supplier_error.clone(),
            });
        }

        if poll.completed {
            if let Some(ref error) = poll.error_message {
                return Err(AixError::OperationFailed {
                    message: error.clone(),
                    supplier_error: poll.supplier_error.clone(),
                });
            }
            return Ok(poll);
        }

        tokio::time::sleep(interval).await;
        interval = Duration::from_secs_f64(
            (interval.as_secs_f64() * config.backoff_factor)
                .max(config.min_interval.as_secs_f64())
                .min(config.max_interval.as_secs_f64()),
        );
    }
}
