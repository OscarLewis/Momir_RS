use crate::OracleScryfallCard;
use crate::ScryfallCard;
use crate::ScryfallClient;
use crate::bulk_data::BulkData;
use crate::bulk_data::oracle_cards::cardset_parser::parse_card_set;
use crate::bulk_data::oracle_cards::filters::OracleFilter;
use crate::bulk_data::oracle_cards::filters::OracleFilters;
use crate::cards::models::ScryfallApiError;
use chrono::{DateTime, NaiveDateTime, Utc};
use rand::prelude::IndexedRandom;
use rand::prelude::IteratorRandom;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tracing::{debug, error};

const ORACLE_CACHE_DIR: &str = "/home/oscar/Documents/Projects/momir_rs_workspace/cache/";

#[derive(Debug, Deserialize, Clone, Default)]
pub struct OracleCards {
    #[serde(default)]
    pub cards: Option<HashMap<String, OracleScryfallCard>>,

    #[serde(skip)]
    creatures_by_cmc: HashMap<u64, HashSet<String>>,

    #[serde(skip)]
    unset_creature_ids: HashSet<String>,
}

impl OracleCards {
    pub async fn new(
        client: &ScryfallClient,
        cache_dir: Option<&PathBuf>,
    ) -> Result<Self, BulkDataError> {
        let bulk_data = BulkData::list(client)
            .await
            .map_err(BulkDataError::Download)?;

        let oracle_cards_bulk = bulk_data
            .iter()
            .find(|bulk_data| bulk_data.data_type == "oracle_cards")
            .ok_or(BulkDataError::OracleCardsNotFound)?
            .clone();

        let target_dir = match cache_dir {
            Some(path) => path.clone(),
            None => PathBuf::from(ORACLE_CACHE_DIR),
        };

        let data_path = write_data(target_dir, oracle_cards_bulk, client).await?;
        let local_card_set = parse_card_set(data_path).await?;

        let unset_creature_ids = ScryfallCard::search(client, "is:unset t:creature")
            .await?
            .into_card_ids()
            .into_iter()
            .collect::<HashSet<_>>();

        debug!(
            num_unset_creatures = unset_creature_ids.len(),
            "Unset creatures fetched from Scryfall"
        );

        let mut creatures_by_cmc: HashMap<u64, HashSet<String>> = HashMap::new();

        for (id, card) in &local_card_set {
            if !card
                .core
                .type_line
                .as_deref()
                .is_some_and(|type_line| type_line.contains("Creature"))
            {
                continue;
            }

            let Some(cmc) = card.core.cmc else {
                continue;
            };

            creatures_by_cmc
                .entry(cmc.to_bits())
                .or_default()
                .insert(card.core.id.clone());
        }

        Ok(Self {
            cards: Some(local_card_set),
            creatures_by_cmc,
            unset_creature_ids,
        })
    }

    pub fn named(&self, name: &str) -> Option<&OracleScryfallCard> {
        self.cards
            .as_ref()?
            .values()
            .find(|card| card.core.name == name)
    }
    pub fn random_creature_by_cmc(
        &self,
        cmc: f64,
        filters: Option<&OracleFilters>,
    ) -> Option<&OracleScryfallCard> {
        let ids = self.creatures_by_cmc.get(&cmc.to_bits())?;

        let eligible = |id: &&String| {
            let Some(filters) = filters else {
                return true;
            };

            filters.filters.iter().all(|filter| {
                match filter {
                    OracleFilter::Unsets => {
                        // Filter out all creature cards that are part of an unset
                        !self.unset_creature_ids.contains(*id)
                    }
                    OracleFilter::Modern => {
                        // TODO: exclude Modern cards
                        true
                    }
                    OracleFilter::Premodern => {
                        // TODO: exclude Premodern cards
                        true
                    }
                    OracleFilter::UnknownEvent => {
                        // TODO: exclude Unknown Event cards
                        true
                    }
                }
            })
        };

        let total_ids = ids.len();
        let eligible_count = ids.iter().filter(eligible).count();

        debug!(
            cmc,
            total_ids,
            eligible_count,
            removed = total_ids - eligible_count,
            "Filtered creature pool"
        );

        let id = ids.iter().filter(eligible).choose(&mut rand::rng())?;

        self.cards.as_ref()?.get(id)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum BulkDataError {
    #[error("failed to read download directory")]
    CreateDirectory(#[source] std::io::Error),

    #[error("failed to get file metadata")]
    Metadata(#[source] std::io::Error),

    #[error("failed to get file modification time")]
    Modified(#[source] std::io::Error),

    #[error("failed to download bulk data")]
    Download(#[source] reqwest::Error),

    #[error("Scryfall API request failed")]
    ScryfallApi(#[from] ScryfallApiError),

    #[error("oracle cards bulk data not found")]
    OracleCardsNotFound,

    #[error("failed to write bulk data")]
    Write(#[source] std::io::Error),

    #[error("failed to read bulk data")]
    Read(#[source] std::io::Error),

    #[error("failed to deserialize Scryfall card")]
    Deserialize(#[source] serde_json::Error),

    #[error("invalid Scryfall updated_at timestamp")]
    InvalidTimestamp(#[source] chrono::ParseError),
}

fn extract_updated_at(path: &Path) -> Option<DateTime<Utc>> {
    let filename = path.file_name()?.to_str()?;

    let timestamp = Regex::new(r"-(\d{14})\.jsonl\.gz$")
        .ok()?
        .captures(filename)?
        .get(1)?
        .as_str();

    let naive = NaiveDateTime::parse_from_str(timestamp, "%Y%m%d%H%M%S").ok()?;

    Some(DateTime::from_naive_utc_and_offset(naive, Utc))
}

async fn write_data(
    download_path: PathBuf,
    bulk_data: BulkData,
    client: &ScryfallClient,
) -> Result<PathBuf, BulkDataError> {
    let mut entries = fs::read_dir(&download_path)
        .await
        .map_err(BulkDataError::CreateDirectory)?;

    let mut files = HashMap::new();

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(BulkDataError::Metadata)?
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let is_jsonl = path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.ends_with(".jsonl.gz"));

        if !is_jsonl {
            continue;
        }

        if let Some(updated_at) = extract_updated_at(&path) {
            files.insert(path, updated_at);
        }
    }

    let most_recent = files.iter().max_by_key(|(_, updated_at)| *updated_at);

    let path = match most_recent {
        None => {
            debug!("No JSONL files found");

            let path = download_path.join(bulk_data.jsonl_download_uri.rsplit('/').next().unwrap());

            debug!("Attempting to download oracle card set from Scryfall...");
            debug!(url = %bulk_data.jsonl_download_uri);

            download_bulk_data(client, &bulk_data.jsonl_download_uri, path.clone()).await?;

            path
        }
        Some((path, updated_at)) => {
            let age = Utc::now() - *updated_at;

            if age > chrono::Duration::days(1) {
                debug!(
                    path = %path.display(),
                    updated_at = %updated_at,
                    "JSONL file is more than one day old"
                );

                let path =
                    download_path.join(bulk_data.jsonl_download_uri.rsplit('/').next().unwrap());

                debug!("Attempting to download oracle card set from Scryfall...");
                debug!(url = %bulk_data.jsonl_download_uri);

                download_bulk_data(client, &bulk_data.jsonl_download_uri, path.clone()).await?;

                path
            } else {
                path.clone()
            }
        }
    };

    // Remove all stale oracle card jsonl.gz files, keeping the file we selected.
    let mut entries = fs::read_dir(&download_path)
        .await
        .map_err(BulkDataError::CreateDirectory)?;

    while let Some(entry) = entries
        .next_entry()
        .await
        .map_err(BulkDataError::Metadata)?
    {
        let candidate = entry.path();

        if candidate == path || !candidate.is_file() {
            continue;
        }

        let is_oracle_jsonl = candidate
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with("oracle-cards-") && name.ends_with(".jsonl.gz"));

        if is_oracle_jsonl {
            debug!(path = %candidate.display(), "Removing old oracle cards file");
            fs::remove_file(&candidate)
                .await
                .map_err(BulkDataError::Metadata)?;
        }
    }

    Ok(path)
}

async fn download_bulk_data(
    client: &ScryfallClient,
    url: &str,
    destination: PathBuf,
) -> Result<(), BulkDataError> {
    debug!(
        url,
        path = %destination.display(),
        "Starting Scryfall bulk-data download"
    );

    let response = client.client.get(url).send().await.map_err(|error| {
        error!(%error, "Scryfall download request failed");
        BulkDataError::Download(error)
    })?;

    debug!(
        status = %response.status(),
        "Received Scryfall download response"
    );

    let bytes = response.bytes().await.map_err(|error| {
        error!(%error, "Failed to read Scryfall download");
        BulkDataError::Download(error)
    })?;

    let mut file = File::create(&destination).await.map_err(|error| {
        error!(
            %error,
            path = %destination.display(),
            "Failed to create download file"
        );
        BulkDataError::Write(error)
    })?;

    file.write_all(&bytes).await.map_err(|error| {
        error!(
            %error,
            path = %destination.display(),
            "Failed to write download"
        );
        BulkDataError::Write(error)
    })?;

    debug!(
        path = %destination.display(),
        size = bytes.len(),
        "Scryfall bulk-data download complete"
    );

    Ok(())
}

// Oracle Card Set API Test Suite
#[cfg(test)]
mod tests {
    use super::*;
    use rand::RngExt;

    fn cache_dir() -> PathBuf {
        PathBuf::from("/home/oscar/Documents/Projects/momir_rs_workspace/cache")
    }

    async fn test_oracle() -> OracleCards {
        let client = ScryfallClient::new().expect("failed to create Scryfall client");

        OracleCards::new(&client, Some(&cache_dir()))
            .await
            .expect("failed to initialize OracleCards")
    }

    #[tokio::test]
    async fn test_oracle_loads_from_cache() {
        let oracle = test_oracle().await;

        let cards = oracle
            .cards
            .as_ref()
            .expect("OracleCards should contain cards");

        assert!(!cards.is_empty());
    }

    #[tokio::test]
    async fn test_named() {
        let oracle = test_oracle().await;

        let card = oracle
            .named("Elvish Mystic")
            .expect("Elvish Mystic should exist in Oracle cards");

        assert_eq!(card.core.name, "Elvish Mystic");
        assert!(
            card.core
                .type_line
                .as_deref()
                .is_some_and(|type_line| type_line.contains("Creature"))
        );
    }

    #[tokio::test]
    async fn test_named_missing_card() {
        let oracle = test_oracle().await;

        assert!(
            oracle
                .named("This Card Definitely Does Not Exist")
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_random_creature_by_cmc() {
        let oracle = test_oracle().await;

        let cmc = rand::rng().random_range(1..=10);

        let card = oracle
            .random_creature_by_cmc(cmc as f64, None)
            .expect("there should be a creature at this CMC");

        assert_eq!(card.core.cmc, Some(cmc as f64));
        assert!(
            card.core
                .type_line
                .as_deref()
                .is_some_and(|type_line| type_line.contains("Creature"))
        );
    }
}
