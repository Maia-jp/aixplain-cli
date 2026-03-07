use serde::{Deserialize, Serialize};

use super::enums::TokenType;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKey {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub budget: Option<f64>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub access_key: Option<String>,
    #[serde(default)]
    pub is_admin: bool,
    #[serde(default)]
    pub global_limits: Option<ApiKeyLimits>,
    #[serde(default)]
    pub assets_limits: Vec<ApiKeyLimits>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyLimits {
    #[serde(rename = "tpm", default)]
    pub token_per_minute: i64,
    #[serde(rename = "tpd", default)]
    pub token_per_day: i64,
    #[serde(rename = "rpm", default)]
    pub request_per_minute: i64,
    #[serde(rename = "rpd", default)]
    pub request_per_day: i64,
    #[serde(rename = "assetId", default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub token_type: Option<TokenType>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyUsage {
    #[serde(rename = "requestCount", default)]
    pub daily_request_count: Option<i64>,
    #[serde(rename = "requestCountLimit", default)]
    pub daily_request_limit: Option<i64>,
    #[serde(rename = "tokenCount", default)]
    pub daily_token_count: Option<i64>,
    #[serde(rename = "tokenCountLimit", default)]
    pub daily_token_limit: Option<i64>,
    #[serde(rename = "assetId", default)]
    pub model_id: Option<String>,
}
