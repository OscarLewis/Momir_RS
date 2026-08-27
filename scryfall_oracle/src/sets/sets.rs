use std::collections::HashSet;

use crate::{ScryfallClient, cards::models::ScryfallApiError};
use reqwest::Url;
use serde::Deserialize;

const SCRYFALL_ALL_SETS: &str = "https://api.scryfall.com/sets";
const SCRYFALL_SET_BY_ID: &str = "https://api.scryfall.com/sets/";

/// Represents the full Catalog object response from Scryfall.
#[derive(Debug, Clone, Deserialize)]
pub struct ScryfallSetListResponse {
    pub object: String,
    pub has_more: bool,
    pub data: Vec<ScryfallSet>,
}

/// A single Set retrieved from the Scryfall API
/// Derived from https://scryfall.com/docs/api/sets
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
    pub released_at: Option<String>,
    pub block_code: Option<String>,
    pub parent_set_code: Option<String>,
    pub card_count: i64,
    pub printed_size: Option<i64>,
    pub digital: bool,
    pub foil_only: bool,
    pub nonfoil_only: bool,
    pub scryfall_uri: String,
    pub uri: String,
    pub icon_svg_uri: String,
    pub search_uri: String,
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

    pub fn get_set_from_id(&self, id: &str) -> Option<&ScryfallSet> {
        self.0.iter().find(|set| set.id == id)
    }

    pub fn get_svg_uri_from_id(&self, id: &str) -> Option<&String> {
        self.0
            .iter()
            .find(|set| set.id == id)
            .map(|set| &set.icon_svg_uri)
    }
}

impl ScryfallSet {
    pub async fn from_id(id: &str, client: &ScryfallClient) -> Result<Self, ScryfallApiError> {
        let mut url = Url::parse(SCRYFALL_SET_BY_ID)?;

        url.path_segments_mut()
            .expect("Scryfall API URL should support path segments")
            .push(id);

        let response = client
            .get(url.as_str(), None)
            .await?
            .error_for_status()?
            .json::<ScryfallSet>()
            .await?;

        Ok(response)
    }

    pub async fn from_code(code: &str, client: &ScryfallClient) -> Result<Self, ScryfallApiError> {
        let mut url = Url::parse(SCRYFALL_SET_BY_ID)?;

        url.path_segments_mut()
            .expect("Scryfall API URL should support path segments")
            .push(code);

        let response = client
            .get(url.as_str(), None)
            .await?
            .error_for_status()?
            .json::<ScryfallSet>()
            .await?;

        Ok(response)
    }

    pub async fn get_svg_bytes(
        &self,
        client: &ScryfallClient,
    ) -> Result<Vec<u8>, ScryfallApiError> {
        Ok(client
            .get(&self.icon_svg_uri, None)
            .await?
            .error_for_status()?
            .bytes()
            .await?
            .to_vec())
    }
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

    #[tokio::test]
    #[ignore]
    async fn test_from_id() {
        let client = ScryfallClient::new().unwrap();

        let set = ScryfallSet::from_id("2ec77b94-6d47-4891-a480-5d0b4e5c9372", &client)
            .await
            .unwrap();

        assert_eq!(set.id, "2ec77b94-6d47-4891-a480-5d0b4e5c9372");
    }

    #[tokio::test]
    #[ignore]
    async fn test_from_code() {
        let client = ScryfallClient::new().unwrap();

        let set = ScryfallSet::from_code("neo", &client).await.unwrap();

        assert_eq!(set.code, "neo");
    }
}
