use crate::layout::Layout;
use image::RgbImage;
use scryfall_oracle::OracleScryfallCard;

pub enum CardType {
    Regular(OracleScryfallCard),
    MDFC(OracleScryfallCard),
    Adventure(OracleScryfallCard),
    Omen(OracleScryfallCard),
    Prepare(OracleScryfallCard),
}

pub trait CardRenderer {
    fn render(
        &self,
        layout: &Layout,
    ) -> impl std::future::Future<Output = Result<RgbImage, Box<dyn std::error::Error>>>;
}
