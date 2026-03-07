use crate::client::polling::{poll_until_complete, PollConfig, PollResponse};
use crate::client::{AixClient, AixError};
use crate::models::common::{Page, PaginateRequest, SortField};
use crate::models::model::{Model, ModelResult};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;

pub struct ModelSearchParams {
    pub query: Option<String>,
    pub functions: Option<Vec<String>>,
    pub suppliers: Option<Vec<String>>,
    pub page: i64,
    pub page_size: i64,
    pub sort: Option<Vec<SortField>>,
}

impl Default for ModelSearchParams {
    fn default() -> Self {
        Self {
            query: None,
            functions: None,
            suppliers: None,
            page: 0,
            page_size: 20,
            sort: None,
        }
    }
}

pub async fn search_models(
    client: &AixClient,
    params: &ModelSearchParams,
) -> Result<Page<Model>, AixError> {
    let sort = params
        .sort
        .clone()
        .unwrap_or_else(|| vec![SortField::default()]);

    let mut req = PaginateRequest {
        q: params.query.clone(),
        page_number: params.page,
        page_size: params.page_size,
        sort: Some(sort),
        ..Default::default()
    };

    if let Some(ref functions) = params.functions {
        let func_filter: Vec<serde_json::Value> =
            functions.iter().map(|f| json!({"id": f})).collect();
        req.filters.insert("functions".into(), json!(func_filter));
    }

    if let Some(ref suppliers) = params.suppliers {
        req.filters.insert("suppliers".into(), json!(suppliers));
    }

    client.post("v2/models/paginate", &req).await
}

pub async fn get_model(client: &AixClient, id: &str) -> Result<Model, AixError> {
    client.get(&format!("v2/models/{id}")).await
}

pub struct ModelInput {
    pub text: Option<String>,
    pub file_url: Option<String>,
    pub params: HashMap<String, serde_json::Value>,
}

pub async fn run_model(
    client: &AixClient,
    id: &str,
    input: &ModelInput,
    timeout_secs: u64,
    on_progress: Option<&dyn Fn(&PollResponse)>,
) -> Result<ModelResult, AixError> {
    let model = get_model(client, id).await?;
    let payload = build_run_payload(input);

    if model.is_sync_only() {
        run_sync(client, id, &payload).await
    } else {
        run_async(client, id, &payload, timeout_secs, on_progress).await
    }
}

fn build_run_payload(input: &ModelInput) -> serde_json::Value {
    let mut payload = serde_json::Map::new();

    if let Some(ref text) = input.text {
        payload.insert("text".into(), json!(text));
    }
    if let Some(ref file_url) = input.file_url {
        payload.insert("fileUrl".into(), json!(file_url));
    }
    for (k, v) in &input.params {
        payload.insert(k.clone(), v.clone());
    }

    serde_json::Value::Object(payload)
}

async fn run_sync(
    client: &AixClient,
    id: &str,
    payload: &serde_json::Value,
) -> Result<ModelResult, AixError> {
    let url = client.models_url(id);
    client.post_url(&url, payload).await
}

async fn run_async(
    client: &AixClient,
    id: &str,
    payload: &serde_json::Value,
    timeout_secs: u64,
    on_progress: Option<&dyn Fn(&PollResponse)>,
) -> Result<ModelResult, AixError> {
    let url = client.models_url(id);
    let response: serde_json::Value = client.post_url(&url, payload).await?;

    let poll_url = extract_poll_url(&response)?;

    let poll_result = poll_until_complete(
        client,
        &poll_url,
        &PollConfig {
            timeout: Duration::from_secs(timeout_secs),
            ..Default::default()
        },
        on_progress,
    )
    .await?;

    Ok(ModelResult {
        status: poll_result.status,
        completed: poll_result.completed,
        error_message: poll_result.error_message,
        supplier_error: poll_result.supplier_error,
        data: poll_result.data,
        run_time: poll_result.run_time,
        used_credits: poll_result.used_credits,
        ..Default::default()
    })
}

fn extract_poll_url(response: &serde_json::Value) -> Result<String, AixError> {
    if let Some(data) = response.get("data") {
        if let Some(url_str) = data.as_str() {
            if url_str.starts_with("http") {
                return Ok(url_str.to_string());
            }
        }
    }
    if let Some(url) = response.get("url").and_then(|u| u.as_str()) {
        return Ok(url.to_string());
    }
    Err(AixError::OperationFailed {
        message: "No polling URL in run response".into(),
        supplier_error: None,
    })
}
