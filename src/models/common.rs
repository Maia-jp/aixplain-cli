use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    #[serde(default)]
    pub results: Vec<T>,
    #[serde(default)]
    pub total: i64,
    #[serde(default)]
    pub page_total: i64,
}

impl<T> Page<T> {
    pub fn is_empty(&self) -> bool {
        self.results.is_empty()
    }

    pub fn has_next(&self, current_page: i64) -> bool {
        current_page + 1 < self.page_total
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OperationResult {
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub completed: bool,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub supplier_error: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginateRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub q: Option<String>,
    #[serde(default)]
    pub page_number: i64,
    #[serde(default = "default_page_size")]
    pub page_size: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ownership: Option<super::enums::Ownership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<Vec<SortField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<String>,
    #[serde(flatten)]
    pub filters: serde_json::Map<String, serde_json::Value>,
}

#[allow(dead_code)]
fn default_page_size() -> i64 {
    20
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SortField {
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub field: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub dir: i32,
}

fn is_zero(v: &i32) -> bool {
    *v == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_deserialize() {
        let json = r#"{"results": [1, 2, 3], "total": 10, "pageTotal": 5}"#;
        let page: Page<i32> = serde_json::from_str(json).unwrap();
        assert_eq!(page.results, vec![1, 2, 3]);
        assert_eq!(page.total, 10);
        assert_eq!(page.page_total, 5);
        assert!(page.has_next(0));
        assert!(!page.has_next(4));
    }

    #[test]
    fn page_empty() {
        let json = r#"{"results": [], "total": 0, "pageTotal": 0}"#;
        let page: Page<String> = serde_json::from_str(json).unwrap();
        assert!(page.is_empty());
    }

    #[test]
    fn paginate_request_models_sort() {
        let req = PaginateRequest {
            q: Some("gpt".into()),
            sort: Some(vec![SortField::default()]),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("sort").is_some());
        assert!(json.get("sortBy").is_none());
    }

    #[test]
    fn paginate_request_agents_sort() {
        let req = PaginateRequest {
            sort_by: Some("NAME".into()),
            sort_order: Some("ASC".into()),
            ..Default::default()
        };
        let json = serde_json::to_value(&req).unwrap();
        assert!(json.get("sort").is_none());
        assert_eq!(json["sortBy"], "NAME");
    }

    #[test]
    fn operation_result_with_data() {
        let json = r#"{
            "status": "SUCCESS",
            "completed": true,
            "data": {"output": "hello"}
        }"#;
        let r: OperationResult = serde_json::from_str(json).unwrap();
        assert!(r.completed);
        assert!(r.data.is_some());
        assert!(r.error_message.is_none());
    }
}
