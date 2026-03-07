use crate::client::{AixClient, AixError};
use crate::models::common::{Page, PaginateRequest};
use crate::models::tool::Tool;

pub async fn search_tools(
    client: &AixClient,
    query: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<Page<Tool>, AixError> {
    let body = PaginateRequest {
        q: query.map(String::from),
        page_number: page,
        page_size,
        ..Default::default()
    };
    client.post("v2/tools/paginate", &body).await
}

pub async fn get_tool(client: &AixClient, id: &str) -> Result<Tool, AixError> {
    client.get(&format!("v2/tools/{id}")).await
}

pub async fn delete_tool(client: &AixClient, id: &str) -> Result<(), AixError> {
    client.delete(&format!("v2/tools/{id}")).await
}
