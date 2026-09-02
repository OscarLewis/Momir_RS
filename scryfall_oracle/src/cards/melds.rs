use std::collections::HashSet;

use tracing::debug;

use crate::{
    OracleScryfallCard, RelatedCard, ScryfallClient,
    cards::models::{ScryfallApiError, ScryfallCardList},
};
#[derive(Debug)]
pub struct OracleMelds {
    pub melds: Vec<OracleMeld>,
}

#[derive(Debug)]
pub struct OracleMeld {
    pub result_id: String, // Result Scryfall ID
    pub children: Vec<OracleMeldChild>,
}

#[derive(Debug)]
pub struct OracleMeldChild {
    pub id: String,   // Child scryfall ID
    pub position: u8, // Is this the first or second card in the meld? used to determine backing art.
}
impl OracleMelds {
    pub async fn get_melds(client: &ScryfallClient) -> Result<OracleMelds, ScryfallApiError> {
        let meld_results = OracleScryfallCard::search(client, "is:meldresult").await?;
        let meld_parts = OracleScryfallCard::search(client, "-is:meldresult layout:meld").await?;

        debug!(
            total_melds = meld_results.total_cards,
            "Fetched meld results from Scryfall"
        );

        let mut melds = Vec::new();

        for card in &meld_results.data {
            let children: Vec<OracleMeldChild> = card
                .core
                .all_parts
                .iter()
                .flatten()
                .filter(|related| related.component == "meld_part")
                .enumerate()
                .map(|(index, related)| OracleMeldChild {
                    id: related.id.clone(),
                    position: (index + 1) as u8,
                })
                .collect();

            debug!(
                result = %card.core.name,
                children = ?children,
                "Found meld parts"
            );

            melds.push(OracleMeld {
                result_id: card.core.id.clone(),
                children,
            });
        }
        let all_melds = OracleMelds { melds };
        debug!(total_melds = all_melds.melds.len(), "All melds");
        Ok(all_melds)
    }
}

// Search for meld results `is:meldresult
// '-is:meldresult layout:meld'  (meld parts)

#[cfg(test)]
mod tests {
    use super::*;
    use test_log::test;

    #[test(tokio::test)]
    async fn test_get_melds() {
        let client = ScryfallClient::new(None).expect("failed to create Scryfall client");

        let melds = OracleMelds::get_melds(&client).await.unwrap();

        assert!(!melds.melds.is_empty());

        for meld in &melds.melds {
            assert!(!meld.result_id.is_empty());
            assert!(!meld.children.is_empty());

            for (index, child) in meld.children.iter().enumerate() {
                assert!(!child.id.is_empty());
                assert_eq!(child.position, (index + 1) as u8);
            }
        }
    }
}
