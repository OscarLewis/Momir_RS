use crate::cards::models::ScryfallApiError;
use crate::sets::sets::ScryfallSets;
use crate::{OracleScryfallCard, ScryfallClient};
use tracing::debug;

const SCRYFALL_CARDS_BY_ID_URL: &str = "https://api.scryfall.com/cards";

impl OracleScryfallCard {
    /// Retrieves a card by its Scryfall ID.
    ///
    /// See: https://scryfall.com/docs/api/cards/id
    pub async fn by_id_live(
        client: &ScryfallClient,
        id: &str,
    ) -> Result<OracleScryfallCard, ScryfallApiError> {
        let response = client
            .get(format!("{SCRYFALL_CARDS_BY_ID_URL}/{id}"), None)
            .await?
            .error_for_status()?;
        let sets = ScryfallSets::new(&client).await?;

        let mut card = response.json::<OracleScryfallCard>().await?;

        let set = sets.get_set_from_id(&card.core.set_id);
        if let Some(set) = set {
            debug!(
                card_id = id,
                card_name = card.core.name,
                set_code = set.code,
                "Scryfall card by ID fetched with set info"
            );
            card.core.set_icon_svg_uri = Some(set.icon_svg_uri.clone());
        }

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

        let result =
            OracleScryfallCard::by_id_live(&client, "08b9a296-3b76-4f8f-9d71-7c9af92bb3b4")
                .await
                .expect("Scryfall lookup failed");

        assert_eq!(result.core.id, "08b9a296-3b76-4f8f-9d71-7c9af92bb3b4");
        assert_eq!(result.core.name, "Elvish Mystic");
    }
}
