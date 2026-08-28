use crate::layout::Layout;
use image::RgbImage;
use scryfall_oracle::OracleScryfallCard;

pub enum CardType {
    Regular(OracleScryfallCard),
    MDFC {
        front: OracleScryfallCard,
        back: OracleScryfallCard,
    },
}

pub trait CardRenderer {
    async fn render(&self, layout: &Layout) -> Result<RgbImage, Box<dyn std::error::Error>>;
}
pub struct RegularCardRenderer<'a> {
    pub card: &'a OracleScryfallCard,
}

impl<'a> CardRenderer for RegularCardRenderer<'a> {
    async fn render(&self, layout: &Layout) -> Result<RgbImage, Box<dyn std::error::Error>> {
        // Your current generate() logic here
        todo!()
    }
}

pub struct MDFCCardRenderer<'a> {
    pub front: &'a OracleScryfallCard,
    pub back: &'a OracleScryfallCard,
}

impl<'a> CardRenderer for MDFCCardRenderer<'a> {
    async fn render(&self, layout: &Layout) -> Result<RgbImage, Box<dyn std::error::Error>> {
        // let front_img = RegularCardRenderer { card: self.front }.render(layout).await?;
        // let back_img = RegularCardRenderer { card: self.back }.render(layout).await?;

        // let mut composed = RgbImage::new(front_img.width() * 2, front_img.height());
        // imageops::overlay(&mut composed, &front_img, 0, 0);
        // imageops::overlay(&mut composed, &back_img, front_img.width() as i64, 0);

        // Ok(composed)
        todo!()
    }
}
