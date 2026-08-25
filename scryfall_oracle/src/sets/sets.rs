use std::collections::HashSet;

use crate::{ScryfallClient, cards::models::ScryfallApiError};
use serde::Deserialize;

const SCRYFALL_ALL_SETS: &str = "https://api.scryfall.com/sets";

/// Represents the full Catalog object response from Scryfall.
#[derive(Debug, Clone, Deserialize)]
pub struct ScryfallSetListResponse {
    pub object: String,
    pub has_more: bool,
    pub data: Vec<ScryfallSet>,
}

#[derive(Debug, Clone, Default)]
pub struct ScryfallSets(HashSet<ScryfallSet>);

impl ScryfallSets {
    pub async fn new(client: &ScryfallClient) -> Result<Self, ScryfallApiError> {
        let response = client
            .get(SCRYFALL_ALL_SETS, None)
            .await?
            .error_for_status()?
            .json::<ScryfallSetListResponse>()
            .await?;

        Ok(Self(response.data.into_iter().collect()))
    }

    pub fn insert(&mut self, set: ScryfallSet) -> bool {
        self.0.insert(set)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ScryfallSet> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
pub struct ScryfallSet {
    pub object: String,
    pub id: String,
    pub code: String,
    pub mtgo_code: Option<String>,
    pub arena_code: Option<String>,
    pub tcgplayer_id: Option<i64>,
    pub name: String,
    pub set_type: ScryfallSetType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScryfallSetType {
    Core,
    Expansion,
    Masters,
    Eternal,
    Alchemy,
    Masterpiece,
    Arsenal,
    FromTheVault,
    Spellbook,
    PremiumDeck,
    DuelDeck,
    DraftInnovation,
    TreasureChest,
    Commander,
    Planechase,
    Archenemy,
    Vanguard,
    Funny,
    Starter,
    Box,
    Promo,
    Token,
    Memorabilia,
    Minigame,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scryfall_sets_contains_commander() {
        let client = ScryfallClient::new().expect("failed to create Scryfall client");

        let sets = ScryfallSets::new(&client)
            .await
            .expect("failed to fetch Scryfall sets");

        assert!(
            sets.iter()
                .any(|set| set.set_type == ScryfallSetType::Commander)
        );
    }
}
