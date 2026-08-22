use serde::Deserialize;

use crate::ScryfallClient;

const BULK_DATA_URL: &str = "https://api.scryfall.com/bulk-data";

#[derive(Debug, Deserialize, Clone)]
pub struct BulkData {
    pub object: String,
    pub id: String,

    #[serde(rename = "type")]
    pub data_type: String,

    pub updated_at: String,
    pub uri: String,
    pub name: String,
    pub description: String,
    pub jsonl_download_uri: String,
    pub compressed_size: u64,
}

#[derive(Debug, Deserialize)]
struct BulkDataResponse {
    object: String,
    has_more: bool,
    data: Vec<BulkData>,
}

impl BulkData {
    pub async fn list(client: &ScryfallClient) -> Result<Vec<BulkData>, reqwest::Error> {
        let response = client
            .client
            .get(BULK_DATA_URL)
            .send()
            .await?
            .error_for_status()?
            .json::<BulkDataResponse>()
            .await?;

        Ok(response.data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fetch_bulk_data() {
        let client = ScryfallClient::new().expect("failed to create Scryfall client");

        let bulk_data = BulkData::list(&client)
            .await
            .expect("failed to fetch bulk data");

        assert!(!bulk_data.is_empty());

        for data in bulk_data {
            assert_eq!(data.object, "bulk_data");
            assert!(!data.id.is_empty());
            assert!(!data.data_type.is_empty());
            assert!(!data.updated_at.is_empty());
            assert!(!data.uri.is_empty());
            assert!(!data.name.is_empty());
            assert!(!data.description.is_empty());
            assert!(!data.jsonl_download_uri.is_empty());
            assert!(data.compressed_size > 0);
        }
    }
}
