use crate::layout::Layout;
use image::RgbImage;
use scryfall_oracle::OracleScryfallCard;

pub enum CardType {
    Regular(OracleScryfallCard),
    MDFC(OracleScryfallCard),
}

pub trait CardRenderer {
    async fn render(&self, layout: &Layout) -> Result<RgbImage, Box<dyn std::error::Error>>;
}
