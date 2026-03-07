use serde::{Deserialize, Serialize};

use super::enums::AssetStatus;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub service_name: Option<String>,
    #[serde(default)]
    pub status: Option<AssetStatus>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub developer: Option<String>,
    #[serde(default)]
    pub vendor: Option<VendorInfo>,
    #[serde(default)]
    pub function: Option<ModelFunction>,
    #[serde(default)]
    pub pricing: Option<Pricing>,
    #[serde(default)]
    pub version: Option<VersionInfo>,
    #[serde(default)]
    pub function_type: Option<String>,
    #[serde(rename = "type", default)]
    pub model_type: Option<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub updated_at: Option<String>,
    #[serde(default)]
    pub supports_streaming: Option<bool>,
    #[serde(default)]
    pub supports_byoc: Option<bool>,
    #[serde(default)]
    pub connection_type: Option<Vec<String>>,
    #[serde(default)]
    pub attributes: Option<Vec<Attribute>>,
    #[serde(default)]
    pub params: Option<Vec<Parameter>>,
}

impl Model {
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or("(unnamed)")
    }

    pub fn is_sync_only(&self) -> bool {
        self.connection_type
            .as_ref()
            .map(|ct| {
                ct.contains(&"synchronous".to_string()) && !ct.contains(&"asynchronous".to_string())
            })
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VendorInfo {
    pub name: Option<String>,
    pub id: Option<serde_json::Value>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFunction {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pricing {
    pub price_per_unit: Option<f64>,
    pub unit_type: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionInfo {
    pub id: Option<String>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attribute {
    pub name: Option<String>,
    pub value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Parameter {
    pub name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub multiple_values: bool,
    #[serde(default)]
    pub is_fixed: bool,
    #[serde(default)]
    pub data_type: Option<String>,
    #[serde(default)]
    pub data_sub_type: Option<String>,
    #[serde(default)]
    pub values: Vec<serde_json::Value>,
    #[serde(default)]
    pub default_values: Vec<serde_json::Value>,
    #[serde(default)]
    pub available_options: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelResult {
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
    pub details: Option<Vec<serde_json::Value>>,
    #[serde(default)]
    pub run_time: Option<f64>,
    #[serde(default)]
    pub used_credits: Option<f64>,
    #[serde(default)]
    pub usage: Option<TokenUsage>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    #[serde(default)]
    pub prompt_tokens: i64,
    #[serde(default)]
    pub completion_tokens: i64,
    #[serde(default)]
    pub total_tokens: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_minimal_deserialize() {
        let json = r#"{"id": "abc123", "name": "GPT-4o"}"#;
        let m: Model = serde_json::from_str(json).unwrap();
        assert_eq!(m.id.as_deref(), Some("abc123"));
        assert_eq!(m.display_name(), "GPT-4o");
        assert!(m.params.is_none());
        assert!(!m.is_sync_only());
    }

    #[test]
    fn model_sync_only() {
        let json = r#"{"connectionType": ["synchronous"]}"#;
        let m: Model = serde_json::from_str(json).unwrap();
        assert!(m.is_sync_only());
    }

    #[test]
    fn model_async_capable() {
        let json = r#"{"connectionType": ["synchronous", "asynchronous"]}"#;
        let m: Model = serde_json::from_str(json).unwrap();
        assert!(!m.is_sync_only());
    }

    #[test]
    fn model_with_params() {
        let json = r#"{
            "id": "m1",
            "params": [
                {"name": "text", "required": true, "dataType": "string"},
                {"name": "temperature", "required": false, "defaultValues": [0.7]}
            ]
        }"#;
        let m: Model = serde_json::from_str(json).unwrap();
        let params = m.params.unwrap();
        assert_eq!(params.len(), 2);
        assert!(params[0].required);
        assert!(!params[1].required);
    }

    #[test]
    fn model_result_deserialize() {
        let json = r#"{
            "status": "SUCCESS",
            "completed": true,
            "data": "The answer is 42",
            "runTime": 1.5,
            "usedCredits": 0.003,
            "usage": {"prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15}
        }"#;
        let r: ModelResult = serde_json::from_str(json).unwrap();
        assert!(r.completed);
        assert_eq!(r.usage.unwrap().total_tokens, 15);
    }

    #[test]
    fn model_unknown_fields_ignored() {
        let json = r#"{"id": "x", "futureField": true, "anotherNew": [1,2,3]}"#;
        let m: Model = serde_json::from_str(json).unwrap();
        assert_eq!(m.id.as_deref(), Some("x"));
    }
}
