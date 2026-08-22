use crate::ScryfallClient;
use crate::bulk_data::BulkData;
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tracing::{debug, error};
const ORACLE_CACHE_DIR: &str = "/home/oscar/Documents/Projects/momir_rs_workspace/cache/";
#[derive(Debug, Deserialize, Clone)]
pub struct OracleCards {}

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

    #[error("oracle cards bulk data not found")]
    OracleCardsNotFound,

    #[error("failed to write bulk data")]
    Write(#[source] std::io::Error),

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

impl OracleCards {
    pub async fn oracle_cards(
        client: &ScryfallClient,
        cache_dir: Option<&PathBuf>,
    ) -> Result<(), BulkDataError> {
        let bulk_data = BulkData::list(client)
            .await
            .map_err(BulkDataError::Download)?;

        let oracle_cards_bulk = bulk_data
            .iter()
            .find(|bulk_data| bulk_data.data_type == "oracle_cards")
            .ok_or(BulkDataError::OracleCardsNotFound)?;

        let target_dir = match cache_dir {
            Some(path) => path.clone(),
            None => PathBuf::from(ORACLE_CACHE_DIR),
        };

        download_bulk_data(&client, &oracle_cards_bulk.jsonl_download_uri, target_dir).await?;
        // Download/write oracle_cards here...
        // TODO Finish and test this
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::ScryfallClient;

    use super::*;

    #[tokio::test]
    async fn test_oracle_cardset_present_in_bulk() {
        let client = ScryfallClient::new().expect("failed to create Scryfall client");

        let bulk_data = BulkData::list(&client)
            .await
            .expect("failed to fetch bulk data");

        let oracle_cards = bulk_data
            .iter()
            .find(|bulk_data| bulk_data.data_type == "oracle_cards")
            .expect("oracle_cards bulk data not found");

        assert_eq!(oracle_cards.object, "bulk_data");
        assert_eq!(oracle_cards.data_type, "oracle_cards");
        assert!(!oracle_cards.id.is_empty());
        assert!(!oracle_cards.updated_at.is_empty());
        assert!(!oracle_cards.uri.is_empty());
        assert!(!oracle_cards.name.is_empty());
        assert!(!oracle_cards.description.is_empty());
        assert!(!oracle_cards.jsonl_download_uri.is_empty());
        assert!(oracle_cards.compressed_size > 0);
    }

    #[tokio::test]
    async fn test_oracle_bulk_integration() {
        let client = ScryfallClient::new().expect("failed to create Scryfall client");
        // let cache = &PathBuf::from("cache");
        let cards = OracleCards::oracle_cards(&client, None).await;
    }
}
