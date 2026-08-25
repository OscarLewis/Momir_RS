use crate::{
    ScryfallClient,
    bulk_data::{BulkData, BulkDataResponse},
};

const BULK_DATA_URL: &str = "https://api.scryfall.com/bulk-data";

impl BulkData {
    pub async fn list(client: &ScryfallClient) -> Result<Vec<BulkData>, reqwest::Error> {
        let response = client
            .get(BULK_DATA_URL, None)
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
