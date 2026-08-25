use crate::{ScryfallClient, cards::models::ScryfallApiError};
use serde::Deserialize;

const SCRYFALL_CATALOG_CREATURE_TYPES: &str = "https://api.scryfall.com/catalog/creature-types";

/// Represents the full Catalog object response from Scryfall.
#[derive(Debug, Clone, Deserialize)]
pub struct ScryfallCatalog {
    pub object: String,
    pub uri: String,
    pub total_values: usize,
    pub data: Vec<String>,
}

impl ScryfallCatalog {
    /// Consumes the catalog and extracts just the data vector.
    pub fn into_data(self) -> Vec<String> {
        self.data
    }

    /// Fetches the full creature types catalog object.
    pub async fn creature_types_catalog(
        client: &ScryfallClient,
    ) -> Result<ScryfallCatalog, ScryfallApiError> {
        let catalog = client
            .get(SCRYFALL_CATALOG_CREATURE_TYPES, None)
            .await?
            .error_for_status()?
            .json::<ScryfallCatalog>()
            .await?;

        Ok(catalog)
    }

    /// Fetches the catalog object and returns only the extracted creature types data vector.
    pub async fn creature_types(client: &ScryfallClient) -> Result<Vec<String>, ScryfallApiError> {
        let catalog = Self::creature_types_catalog(client).await?;
        Ok(catalog.into_data())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::OnceLock;

    fn test_client() -> &'static ScryfallClient {
        static CLIENT: OnceLock<ScryfallClient> = OnceLock::new();

        CLIENT.get_or_init(|| ScryfallClient::new().expect("failed to create Scryfall client"))
    }

    #[tokio::test]
    async fn test_live_creature_types() {
        let catalog = ScryfallCatalog::creature_types_catalog(test_client())
            .await
            .expect("Failed to fetch creature types catalog from Scryfall");

        assert_eq!(
            catalog.data.len(),
            catalog.total_values,
            "Data vector length ({}) does not match total_values ({})",
            catalog.data.len(),
            catalog.total_values
        );

        assert!(
            catalog.data.iter().any(|t| t == "Elf"),
            "Expected 'Elf' to be present in Scryfall creature types catalog"
        );
    }
}
