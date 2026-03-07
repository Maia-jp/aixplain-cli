use serde::{Deserialize, Serialize};

use super::model::Model;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    #[serde(flatten)]
    pub model: Model,
    #[serde(default)]
    pub asset_id: Option<String>,
    #[serde(default)]
    pub allowed_actions: Option<Vec<String>>,
    #[serde(default)]
    pub subscriptions: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_deserialize() {
        let json =
            r#"{"id": "t1", "name": "My Tool", "assetId": "asset1", "allowedActions": ["SEARCH"]}"#;
        let t: Tool = serde_json::from_str(json).unwrap();
        assert_eq!(t.model.id.as_deref(), Some("t1"));
        assert_eq!(t.asset_id.as_deref(), Some("asset1"));
        assert_eq!(t.allowed_actions.as_ref().unwrap(), &["SEARCH"]);
    }
}
