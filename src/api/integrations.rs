use crate::client::{AixClient, AixError};
use crate::models::common::{Page, PaginateRequest};
use crate::models::integration::Integration;

pub async fn search_integrations(
    client: &AixClient,
    query: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<Page<Integration>, AixError> {
    let body = PaginateRequest {
        q: query.map(String::from),
        page_number: page,
        page_size,
        ..Default::default()
    };
    client.post("v2/integrations/paginate", &body).await
}

pub async fn get_integration(client: &AixClient, id: &str) -> Result<Integration, AixError> {
    client.get(&format!("v2/integrations/{id}")).await
}
