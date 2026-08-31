use tracing::debug;

use crate::{
    OracleScryfallCard, ScryfallClient,
    cards::models::{ScryfallApiError, ScryfallCardList},
};

pub struct OracleMelds {
    meld_results: Vec<OracleScryfallCard>,
    meld_parts: Vec<OracleScryfallCard>,
}

impl OracleMelds {
    // TODO Deal with Meld cards by allowing users to access OracleScryfallCard for each of the melded cards.
    // Associate the result Cards halves with each original piece.
    // Cool but a pita.

    pub async fn get_melds(client: &ScryfallClient) -> Result<OracleMelds, ScryfallApiError> {
        let meld_results = OracleScryfallCard::search(client, "is:meldresult").await?;
        let meld_parts = OracleScryfallCard::search(client, "-is:meldresult layout:meld").await?;
        debug!(
            total_melds = meld_results.total_cards,
            "Fetched meld results from Scryfall"
        );
        Ok(OracleMelds {
            meld_results: meld_results.data,
            meld_parts: meld_parts.data,
        })
    }
}

// Search for meld results `is:meldresult
#[cfg(test)]
mod tests {
    use super::*;
    use crate::CardLayout;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_get_melds() {
        let client = ScryfallClient::new(None).expect("failed to create Scryfall client");

        let melds = OracleMelds::get_melds(&client).await.unwrap();

        assert!(!melds.meld_results.is_empty());
        assert!(!melds.meld_parts.is_empty());

        for meld in &melds.meld_results {
            assert!(
                matches!(meld.core.layout, CardLayout::Meld),
                "Expected meld card, got {}",
                meld.core.name
            );
        }
        for meld in &melds.meld_parts {
            assert!(
                matches!(meld.core.layout, CardLayout::Meld),
                "Expected meld card, got {}",
                meld.core.name
            );
        }
    }
}

// '-is:meldresult layout:meld'  (meld parts)
