use crate::{ScryfallCard, ScryfallClient};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::debug;

const SCRYFALL_SEARCH_URL: &str = "https://api.scryfall.com/cards/search";

/// Low-level page response returned directly by the Scryfall API.
#[derive(Debug, Deserialize)]
pub struct ScryfallPageResponse<T> {
    pub object: String,
    pub total_cards: Option<u32>,
    pub has_more: bool,
    pub next_page: Option<String>,
    pub data: Vec<T>,
}

/// Fully-aggregated list containing all pages from a search query.
#[derive(Debug, Clone)]
pub struct ScryfallCardList {
    pub total_cards: Option<u32>,
    pub data: Vec<ScryfallCard>,
}

impl ScryfallCardList {
    /// Extracts a list of `id` strings from `card.core.id` for all cards.
    pub fn card_ids(&self) -> Vec<String> {
        self.data.iter().map(|card| card.core.id.clone()).collect()
    }

    /// Consumes the list and extracts all `card.core.id` strings without cloning.
    pub fn into_card_ids(self) -> Vec<String> {
        self.data.into_iter().map(|card| card.core.id).collect()
    }
}

impl ScryfallCard {
    /// Executes a search query, following all `next_page` pagination links to collect every card.
    pub async fn search(
        client: &ScryfallClient,
        query: &str,
    ) -> Result<ScryfallCardList, reqwest::Error> {
        let mut params = HashMap::new();
        params.insert("q", query);

        let mut all_cards = Vec::new();

        // First page fetch using query parameters
        let response = client
            .client
            .get(SCRYFALL_SEARCH_URL)
            .query(&params)
            .send()
            .await?
            .error_for_status()?;

        let mut page: ScryfallPageResponse<ScryfallCard> = response.json().await?;
        let total_cards = page.total_cards;
        all_cards.append(&mut page.data);

        // Follow next_page URIs until has_more is false
        while page.has_more {
            if let Some(next_url) = page.next_page {
                let next_response = client
                    .client
                    .get(&next_url)
                    .send()
                    .await?
                    .error_for_status()?;

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
