use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct BulkData {
    pub object: String,
    pub id: String,

    #[serde(rename = "type")]
    pub data_type: String,

    pub updated_at: String,
    pub uri: String,
    pub name: String,
    pub description: String,
    pub jsonl_download_uri: String,
    pub compressed_size: u64,
}

#[derive(Debug, Deserialize)]
pub struct BulkDataResponse {
    pub(crate) object: String,
    pub(crate) has_more: bool,
    pub(crate) data: Vec<BulkData>,
}
