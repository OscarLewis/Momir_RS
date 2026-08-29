use crate::cards::models::ScryfallApiError;
use crate::{ScryfallCard, ScryfallClient};
use tracing::debug;

const SCRYFALL_CARDS_BY_ID_URL: &str = "https://api.scryfall.com/cards";

impl ScryfallCard {
    /// Retrieves a card by its Scryfall ID.
    ///
    /// See: https://scryfall.com/docs/api/cards/id
    pub async fn by_id(
        client: &ScryfallClient,
        id: &str,
    ) -> Result<ScryfallCard, ScryfallApiError> {
        let response = client
            .get(format!("{SCRYFALL_CARDS_BY_ID_URL}/{id}"), None)
            .await?
            .error_for_status()?;

        let card = response.json::<ScryfallCard>().await?;

        debug!(
            card_id = id,
            card_name = card.core.name,
            "Scryfall card by ID fetched"
        );

        Ok(card)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn by_id_returns_card() {
        let client = ScryfallClient::new(None).expect("failed to create Scryfall client");

        let result = ScryfallCard::by_id(&client, "08b9a296-3b76-4f8f-9d71-7c9af92bb3b4")
            .await
            .expect("Scryfall lookup failed");

        assert_eq!(result.core.id, "08b9a296-3b76-4f8f-9d71-7c9af92bb3b4");
        assert_eq!(result.core.name, "Elvish Mystic");
    }
}
