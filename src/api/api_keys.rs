use crate::client::{AixClient, AixError};
use crate::models::api_key::{ApiKey, ApiKeyUsage};

pub async fn list_api_keys(client: &AixClient) -> Result<Vec<ApiKey>, AixError> {
    client.get("sdk/api-keys").await
}

pub async fn get_api_key(client: &AixClient, id: &str) -> Result<ApiKey, AixError> {
    client.get(&format!("sdk/api-keys/{id}")).await
}

pub async fn get_usage(client: &AixClient, id: Option<&str>) -> Result<Vec<ApiKeyUsage>, AixError> {
    let path = match id {
        Some(key_id) => format!("sdk/api-keys/{key_id}/usage-limits"),
        None => "sdk/api-keys/usage-limits".to_string(),
    };
    client.get(&path).await
}

pub async fn delete_api_key(client: &AixClient, id: &str) -> Result<(), AixError> {
    client.delete(&format!("sdk/api-keys/{id}")).await
}
