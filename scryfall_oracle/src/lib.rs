use reqwest::Client;
use serde::Deserialize;

const SCRYFALL_BULK_DATA_URL: &str = "https://api.scryfall.com/bulk-data";

#[derive(Debug, Deserialize)]
pub struct BulkDataResponse {
    pub object: String,
    pub has_more: bool,
    pub data: Vec<BulkData>,
}

#[derive(Debug, Deserialize)]
pub struct BulkData {
    pub object: String,
    pub id: String,

    #[serde(rename = "type")]
    pub kind: String,

    pub updated_at: String,
    pub uri: String,
    pub name: String,
    pub description: String,
    pub jsonl_download_uri: String,
    pub compressed_size: u64,
}

pub async fn fetch_bulk_data(client: &Client) -> Result<BulkDataResponse, reqwest::Error> {
    client
        .get(SCRYFALL_BULK_DATA_URL)
        .send()
        .await?
        .error_for_status()?
        .json::<BulkDataResponse>()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::{
        Client,
        header::{ACCEPT, HeaderMap, HeaderValue, USER_AGENT},
    };

    #[tokio::test]
    async fn fetch_bulk_data_from_scryfall() {
        let mut headers = HeaderMap::new();

        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));

        headers.insert(USER_AGENT, HeaderValue::from_static("momir_rs/0.1.0"));

        let client = Client::builder()
            .default_headers(headers)
            .build()
            .expect("failed to create test client");

        let response = fetch_bulk_data(&client)
            .await
            .expect("failed to fetch Scryfall bulk data");

        assert_eq!(response.object, "list");
        assert!(!response.has_more);
        assert!(!response.data.is_empty());

        for bulk_data in &response.data {
            assert_eq!(bulk_data.object, "bulk_data");
            assert!(!bulk_data.id.is_empty());
            assert!(!bulk_data.name.is_empty());
            assert!(!bulk_data.description.is_empty());
            assert!(!bulk_data.uri.is_empty());
            assert!(!bulk_data.jsonl_download_uri.is_empty());
            assert!(bulk_data.compressed_size > 0);
        }
    }
}
