use serde::{Deserialize, Serialize};

use super::enums::AssetStatus;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub status: Option<AssetStatus>,
    #[serde(default)]
    pub team_id: Option<i64>,
    #[serde(default)]
    pub tools: Option<Vec<AgentToolRef>>,
    #[serde(default)]
    pub inspector_id: Option<String>,
    #[serde(default)]
    pub supervisor_id: Option<String>,
    #[serde(default)]
    pub planner_id: Option<String>,
    #[serde(default)]
    pub tasks: Option<Vec<AgentTask>>,
    #[serde(rename = "agents", default)]
    pub subagents: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub output_format: Option<String>,
    #[serde(default)]
    pub expected_output: Option<serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub max_iterations: Option<i32>,
    #[serde(default)]
    pub max_tokens: Option<i32>,
    #[serde(default)]
    pub resource_info: Option<serde_json::Value>,
    #[serde(default)]
    pub inspector_targets: Option<Vec<String>>,
    #[serde(default)]
    pub max_inspectors: Option<i32>,
    #[serde(default)]
    pub inspectors: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentToolRef {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(rename = "assetId", default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub supplier: Option<String>,
    #[serde(default)]
    pub function: Option<String>,
    #[serde(rename = "type", default)]
    pub tool_type: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub parameters: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub actions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentTask {
    pub name: String,
    #[serde(rename = "description", default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub expected_output: Option<String>,
    #[serde(default)]
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunRequest {
    pub id: String,
    pub query: AgentQuery,
    #[serde(default)]
    pub execution_params: AgentExecutionParams,
    #[serde(default = "crate::models::agent::default_true")]
    pub run_response_generation: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Vec<AgentTask>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inspectors: Option<Vec<serde_json::Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub history: Option<Vec<ChatMessage>>,
    #[serde(default = "crate::models::agent::default_true")]
    pub allow_history_and_session_id: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub criteria: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evolve: Option<String>,
}

fn default_true() -> bool {
    true
}

impl Default for AgentRunRequest {
    fn default() -> Self {
        Self {
            id: String::new(),
            query: AgentQuery::default(),
            execution_params: AgentExecutionParams::default(),
            run_response_generation: true,
            tasks: None,
            inspectors: None,
            session_id: None,
            history: None,
            allow_history_and_session_id: true,
            prompt: None,
            criteria: None,
            evolve: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AgentQuery {
    pub input: String,
    #[serde(flatten)]
    pub variables: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentExecutionParams {
    #[serde(default = "default_output_format")]
    pub output_format: String,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: i32,
    #[serde(default = "default_max_iterations")]
    pub max_iterations: i32,
    #[serde(default = "default_max_time")]
    pub max_time: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expected_output: Option<serde_json::Value>,
}

impl Default for AgentExecutionParams {
    fn default() -> Self {
        Self {
            output_format: default_output_format(),
            max_tokens: default_max_tokens(),
            max_iterations: default_max_iterations(),
            max_time: default_max_time(),
            expected_output: None,
        }
    }
}

fn default_output_format() -> String {
    "text".to_string()
}
fn default_max_tokens() -> i32 {
    2048
}
fn default_max_iterations() -> i32 {
    5
}
fn default_max_time() -> i32 {
    300
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunResult {
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
    pub session_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    #[serde(default)]
    pub used_credits: Option<f64>,
    #[serde(default)]
    pub run_time: Option<f64>,
}

impl AgentRunResult {
    pub fn output_text(&self) -> String {
        if let Some(ref data) = self.data {
            if let Some(output) = data.get("output") {
                if let Some(s) = output.as_str() {
                    return s.to_string();
                }
                return serde_json::to_string_pretty(output).unwrap_or_default();
            }
            return serde_json::to_string_pretty(data).unwrap_or_default();
        }
        "(no output)".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_minimal() {
        let json = r#"{"id": "a1", "name": "Test Agent"}"#;
        let a: Agent = serde_json::from_str(json).unwrap();
        assert_eq!(a.id.as_deref(), Some("a1"));
        assert!(a.tools.is_none());
        assert!(a.max_iterations.is_none());
    }

    #[test]
    fn agent_with_tools() {
        let json = r#"{
            "id": "a1",
            "tools": [{"assetId": "t1", "name": "Search", "type": "model"}]
        }"#;
        let a: Agent = serde_json::from_str(json).unwrap();
        let tools = a.tools.unwrap();
        assert_eq!(tools[0].asset_id.as_deref(), Some("t1"));
    }

    #[test]
    fn agent_run_request_serializes() {
        let req = AgentRunRequest {
            id: "a1".into(),
            query: AgentQuery {
                input: "Hello".into(),
                variables: Default::default(),
            },
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert_eq!(json["id"], "a1");
        assert_eq!(json["query"]["input"], "Hello");
        assert_eq!(json["executionParams"]["maxTokens"], 2048);
        assert_eq!(json["executionParams"]["maxIterations"], 5);
        assert_eq!(json["runResponseGeneration"], true);
        assert_eq!(json["allowHistoryAndSessionId"], true);
    }

    #[test]
    fn agent_run_result_output() {
        let json = r#"{"data": {"output": "Hello world"}, "status": "SUCCESS", "completed": true}"#;
        let r: AgentRunResult = serde_json::from_str(json).unwrap();
        assert_eq!(r.output_text(), "Hello world");
    }

    #[test]
    fn agent_subagents_field_rename() {
        let json = r#"{"id": "a1", "agents": [{"id": "sub1"}]}"#;
        let a: Agent = serde_json::from_str(json).unwrap();
        assert!(a.subagents.is_some());
        assert_eq!(a.subagents.unwrap().len(), 1);
    }
}
