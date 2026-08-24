use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ==========================================
// 1. SHARED CORE FIELDS (Present in both API & Bulk)
// ==========================================

/// Core identity & gameplay metadata common across all card data formats
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CoreCardFields {
    pub id: String,
    pub oracle_id: Option<String>,
    pub name: String,
    pub lang: String,
    pub released_at: Option<String>,
    pub uri: String,
    pub scryfall_uri: String,
    pub layout: String,

    // Gameplay
    pub mana_cost: Option<String>,
    pub cmc: Option<f64>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defense: Option<String>,
    pub colors: Option<Vec<String>>,
    pub color_identity: Vec<String>,
    pub color_indicator: Option<Vec<String>>,
    pub keywords: Vec<String>,
    pub legalities: HashMap<String, String>,
    pub reserved: bool,
    pub game_changer: Option<bool>,

    // Set Info (Canonical set data)
    pub set_id: String,
    pub set: String,
    pub set_name: String,
    pub set_type: String,
    pub set_uri: String,
    pub set_search_uri: String,
    pub scryfall_set_uri: String,
    pub rulings_uri: String,
    pub prints_search_uri: String,

    // Auxiliary / Relations
    pub all_parts: Option<Vec<RelatedCard>>,
    pub card_faces: Option<Vec<CardFace>>,
}

// ==========================================
// 2. SHARED PRINT FIELDS
// ==========================================

/// Visual and physical printing details shared by full card objects
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrintFields {
    pub multiverse_ids: Option<Vec<i64>>,
    pub mtgo_id: Option<i64>,
    pub mtgo_foil_id: Option<i64>,
    pub tcgplayer_id: Option<i64>,
    pub cardmarket_id: Option<i64>,

    pub highres_image: bool,
    pub image_status: String,
    pub image_updated_at: Option<String>,
    pub image_uris: Option<ImageUris>,

    pub collector_number: String,
    pub digital: bool,
    pub rarity: String,
    pub watermark: Option<String>,
    pub flavor_text: Option<String>,
    pub card_back_id: Option<String>,
    pub artist: Option<String>,
    pub artist_ids: Option<Vec<String>>,
    pub illustration_id: Option<String>,
    pub border_color: String,
    pub frame: String,
    pub frame_effects: Option<Vec<String>>,
    pub security_stamp: Option<String>,

    pub full_art: bool,
    pub textless: bool,
    pub booster: bool,
    pub story_spotlight: bool,
    pub edhrec_rank: Option<i64>,
    pub penny_rank: Option<i64>,

    pub games: Vec<String>,
    pub finishes: Vec<String>,
    pub foil: bool,
    pub nonfoil: bool,
    pub oversized: bool,
    pub promo: bool,
    pub reprint: bool,
    pub variation: bool,

    pub prices: Prices,
    pub related_uris: HashMap<String, String>,
    pub purchase_uris: Option<HashMap<String, String>>,
    pub preview: Option<PreviewInfo>,
}

// ==========================================
// 3. COMPOSITE CARDS
// ==========================================

/// Full Scryfall Card (Default API Endpoint & 'Default Cards' Bulk JSONL File)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScryfallCard {
    #[serde(flatten)]
    pub core: CoreCardFields,

    #[serde(flatten)]
    pub print: PrintFields,
}

/// Oracle Scryfall Card ('Oracle Cards' Bulk JSONL File)
/// Shares core and print data via composition, keeping code DRY.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OracleScryfallCard {
    #[serde(flatten)]
    pub core: CoreCardFields,

    #[serde(flatten)]
    pub print: PrintFields,
}

// ==========================================
// 4. NESTED TYPES
// ==========================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ImageUris {
    pub small: Option<String>,
    pub normal: Option<String>,
    pub large: Option<String>,
    pub png: Option<String>,
    pub art_crop: Option<String>,
    pub border_crop: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Prices {
    pub usd: Option<String>,
    pub usd_foil: Option<String>,
    pub usd_etched: Option<String>,
    pub eur: Option<String>,
    pub eur_foil: Option<String>,
    pub tix: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RelatedCard {
    pub object: String,
    pub id: String,
    pub component: String,
    pub name: String,
    pub type_line: String,
    pub uri: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CardFace {
    pub object: String,
    pub name: String,
    pub mana_cost: Option<String>,
    pub type_line: Option<String>,
    pub oracle_text: Option<String>,
    pub colors: Option<Vec<String>>,
    pub power: Option<String>,
    pub toughness: Option<String>,
    pub loyalty: Option<String>,
    pub defense: Option<String>,
    pub flavor_text: Option<String>,
    pub illustration_id: Option<String>,
    pub image_uris: Option<ImageUris>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreviewInfo {
    pub source: Option<String>,
    pub source_uri: Option<String>,
    pub previewed_at: Option<String>,
}
