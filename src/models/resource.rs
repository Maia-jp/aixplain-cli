use serde::{Deserialize, Serialize};

use super::enums::FileType;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    #[serde(default)]
    pub file_path: Option<String>,
    #[serde(default)]
    pub s3_url: Option<String>,
    #[serde(default)]
    pub file_type: Option<FileType>,
    #[serde(default)]
    pub is_temp: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PresignedUrlResponse {
    pub key: String,
    pub upload_url: String,
    #[serde(default)]
    pub download_url: Option<String>,
}
