use crate::bulk_data::oracle_cards::OracleScryfallCard;
use crate::bulk_data::oracle_cards::oracle::BulkDataError;
use async_compression::tokio::bufread::GzipDecoder;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};
use tracing::debug;

pub async fn parse_card_set(
    path: PathBuf,
) -> Result<HashMap<String, OracleScryfallCard>, BulkDataError> {
    debug!(
        path = %path.display(),
        "Preparing to parse Scryfall card set located at"
    );

    let file = File::open(&path).await.map_err(BulkDataError::Read)?;

    let buf_reader = BufReader::new(file);
    let decoder = GzipDecoder::new(buf_reader);
    let mut lines = BufReader::new(decoder).lines();

    let mut count = 0;
    let mut scryfall_cards = HashMap::new();

    while let Some(line) = lines.next_line().await.map_err(BulkDataError::Read)? {
        if line.trim().is_empty() {
            continue;
        }

        let card = serde_json::from_str::<OracleScryfallCard>(&line)
            .map_err(BulkDataError::Deserialize)?;

        count += 1;

        if card
            .type_line
            .as_deref()
            .is_some_and(|type_line| type_line.contains("Creature"))
        {
            scryfall_cards.insert(card.id.clone(), card);
        }
    }

    debug!(
        total_parsed = count,
        "Successfully finished parsing Scryfall card set"
    );

    Ok(scryfall_cards)
}
