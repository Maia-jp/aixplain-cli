use serde::{Deserialize, Serialize};

use super::model::Model;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Integration {
    #[serde(flatten)]
    pub model: Model,
    #[serde(default)]
    pub actions_available: Option<bool>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub available_versions: Option<Vec<String>>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub input_parameters: Option<serde_json::Value>,
    #[serde(default)]
    pub output_parameters: Option<serde_json::Value>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
    #[serde(default)]
    pub no_auth: Option<bool>,
    #[serde(default)]
    pub inputs: Option<Vec<ActionInput>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionInput {
    pub name: String,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub value: Vec<serde_json::Value>,
    #[serde(default)]
    pub available_options: Vec<serde_json::Value>,
    #[serde(default = "default_datatype")]
    pub datatype: String,
    #[serde(default)]
    pub allow_multi: bool,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub description: String,
}

fn default_datatype() -> String {
    "string".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integration_deserialize() {
        let json = r#"{"id": "i1", "name": "GitHub", "actionsAvailable": true}"#;
        let i: Integration = serde_json::from_str(json).unwrap();
        assert_eq!(i.model.id.as_deref(), Some("i1"));
        assert_eq!(i.actions_available, Some(true));
    }
}
