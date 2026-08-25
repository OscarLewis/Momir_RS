use crate::{
    ScryfallCard, ScryfallClient,
    cards::models::{ScryfallApiError, ScryfallCardList, ScryfallPageResponse},
};
use std::collections::HashMap;
use tracing::debug;

const SCRYFALL_SEARCH_URL: &str = "https://api.scryfall.com/cards/search";

impl ScryfallCard {
    /// Executes a search query, following all `next_page` pagination links to collect every card
    pub async fn search(
        client: &ScryfallClient,
        query: &str,
    ) -> Result<ScryfallCardList, ScryfallApiError> {
        let mut params = HashMap::new();
        params.insert("q", query);

        // Gotta keep all these cards somewhere, might as well be here
        let mut all_cards = Vec::new();

        let response = client
            .get(SCRYFALL_SEARCH_URL, Some(&params))
            .await?
            .error_for_status()?;

        let mut page: ScryfallPageResponse<ScryfallCard> = response.json().await?;
        let total_cards = page.total_cards;
        all_cards.append(&mut page.data);

        while page.has_more {
            if let Some(next_url) = page.next_page {
                let next_response = client.get(&next_url, None).await?.error_for_status()?;

                page = next_response.json().await?;
                all_cards.append(&mut page.data);
            } else {
                break;
            }
        }

        debug!(
            total_cards = total_cards,
            query = query,
            "Scryfall search for query resulted in num cards"
        );

        Ok(ScryfallCardList {
            total_cards,
            data: all_cards,
        })
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn search_returns_matching_cards() {
        let client = ScryfallClient::new().expect("failed to create Scryfall client");

        let result = ScryfallCard::search(&client, "is:unset")
            .await
            .expect("Scryfall search failed");

        assert!(!result.data.is_empty(), "expected search to return cards");

        let ids = result.card_ids();
        assert_eq!(ids.len(), result.data.len());
        assert!(ids.iter().all(|id| !id.is_empty()));
    }
}
