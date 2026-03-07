use crate::client::polling::{poll_until_complete, PollConfig};
use crate::client::{AixClient, AixError};
use crate::models::agent::{
    Agent, AgentExecutionParams, AgentQuery, AgentRunRequest, AgentRunResult,
};
use crate::models::common::{Page, PaginateRequest};
use serde_json::json;
use std::time::Duration;

pub async fn search_agents(
    client: &AixClient,
    query: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<Page<Agent>, AixError> {
    let body = PaginateRequest {
        q: query.map(String::from),
        page_number: page,
        page_size,
        ..Default::default()
    };
    client.post("v2/agents/paginate", &body).await
}

pub async fn get_agent(client: &AixClient, id: &str) -> Result<Agent, AixError> {
    client.get(&format!("v2/agents/{id}")).await
}

pub async fn create_agent(
    client: &AixClient,
    name: &str,
    instructions: Option<&str>,
    llm_id: Option<&str>,
    tool_ids: &[String],
    max_iterations: i32,
    max_tokens: i32,
) -> Result<Agent, AixError> {
    let tools: Vec<serde_json::Value> = tool_ids.iter().map(|id| json!({"assetId": id})).collect();

    let llm = llm_id.unwrap_or("669a63646eb56306647e1091");

    let body = json!({
        "name": name,
        "description": instructions,
        "instructions": instructions,
        "model": {"id": llm},
        "tools": tools,
        "status": "onboarded",
        "maxIterations": max_iterations,
        "maxTokens": max_tokens,
        "outputFormat": "text",
        "agents": [],
        "tasks": [],
    });

    client.post("v2/agents", &body).await
}

pub async fn update_agent(
    client: &AixClient,
    id: &str,
    name: Option<&str>,
    instructions: Option<&str>,
    llm_id: Option<&str>,
    tool_ids: Option<&[String]>,
) -> Result<Agent, AixError> {
    let mut current: serde_json::Value = client.get(&format!("v2/agents/{id}")).await?;
    let obj = current
        .as_object_mut()
        .ok_or_else(|| AixError::OperationFailed {
            message: "Invalid agent response".into(),
            supplier_error: None,
        })?;

    if let Some(n) = name {
        obj.insert("name".into(), json!(n));
    }
    if let Some(inst) = instructions {
        obj.insert("instructions".into(), json!(inst));
        obj.insert("description".into(), json!(inst));
    }
    if let Some(llm) = llm_id {
        obj.insert("model".into(), json!({"id": llm}));
    }
    if let Some(tools) = tool_ids {
        let t: Vec<serde_json::Value> = tools.iter().map(|id| json!({"assetId": id})).collect();
        obj.insert("tools".into(), json!(t));
    }

    client.put(&format!("v2/agents/{id}"), &current).await
}

pub async fn delete_agent(client: &AixClient, id: &str) -> Result<(), AixError> {
    client.delete(&format!("v2/agents/{id}")).await
}

pub async fn run_agent(
    client: &AixClient,
    id: &str,
    query: &str,
    session_id: Option<&str>,
    output_format: &str,
    timeout_secs: u64,
    on_progress: Option<&(dyn Fn(&str) + Send + Sync)>,
) -> Result<AgentRunResult, AixError> {
    let body = AgentRunRequest {
        id: id.to_string(),
        query: AgentQuery {
            input: query.to_string(),
            variables: Default::default(),
        },
        execution_params: AgentExecutionParams {
            output_format: output_format.to_string(),
            max_time: timeout_secs as i32,
            ..Default::default()
        },
        session_id: session_id.map(String::from),
        ..Default::default()
    };

    let response: serde_json::Value = client.post(&format!("v2/agents/{id}/run"), &body).await?;

    let poll_url = extract_poll_url(&response)?;

    let poll_cb = on_progress.map(|cb| {
        move |resp: &crate::client::polling::PollResponse| {
            cb(&resp.status);
        }
    });

    let result = poll_until_complete(
        client,
        &poll_url,
        &PollConfig {
            timeout: Duration::from_secs(timeout_secs),
            ..Default::default()
        },
        poll_cb.as_ref().map(|f| f as &(dyn Fn(&_) + Send + Sync)),
    )
    .await?;

    Ok(AgentRunResult {
        status: result.status,
        completed: result.completed,
        error_message: result.error_message,
        supplier_error: result.supplier_error,
        data: result.data,
        session_id: result.session_id,
        request_id: result.request_id,
        used_credits: result.used_credits,
        run_time: result.run_time,
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
        message: "No polling URL in agent run response".into(),
        supplier_error: None,
    })
}
