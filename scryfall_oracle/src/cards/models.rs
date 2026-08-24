use serde::Deserialize;

use crate::ScryfallCard;

#[derive(Debug, thiserror::Error)]
pub enum ScryfallApiError {
    #[error("Scryfall API request failed: {0}")]
    Reqwest(#[from] reqwest::Error),
}

/// Low-level page response returned directly by the Scryfall API
#[derive(Debug, Deserialize)]
pub struct ScryfallPageResponse<T> {
    pub object: String,
    pub total_cards: Option<u32>,
    pub has_more: bool,
    pub next_page: Option<String>,
    pub data: Vec<T>,
}

/// Fully-aggregated list containing all pages from a search query
#[derive(Debug, Clone)]
pub struct ScryfallCardList {
    pub total_cards: Option<u32>,
    pub data: Vec<ScryfallCard>,
}

impl ScryfallCardList {
    /// Extracts a list of `id` strings from `card.core.id` for all cards with cloning
    pub fn card_ids(&self) -> Vec<String> {
        self.data.iter().map(|card| card.core.id.clone()).collect()
    }

    /// Consumes the list and extracts all `card.core.id` strings without cloning.
    pub fn into_card_ids(self) -> Vec<String> {
        self.data.into_iter().map(|card| card.core.id).collect()
    }
}
