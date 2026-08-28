use crate::{
    card::card_type::{CardRenderer, CardType},
    card::element_renderers::{
        ArtistRenderer, BorderRenderer, CardArtRenderer, ElementRenderer, ManaCostRenderer,
        NameRenderer, OracleTextRenderer, PowerToughnessRenderer, SetCodeRenderer, SetIconRenderer,
        TypeLineRenderer,
    },
    layout::Layout,
};
use image::{Rgb, RgbImage, imageops};
use scryfall_oracle::OracleScryfallCard;
use std::path::PathBuf;
use tracing::info;

/// Handles rendering a single card face
async fn render_card_face(
    card: &OracleScryfallCard,
    layout: &Layout,
) -> Result<RgbImage, Box<dyn std::error::Error>> {
    let mut card_img = RgbImage::from_pixel(layout.width, layout.height, Rgb([255, 255, 255]));
    let mut layout = layout.clone();

    let scryfall_id = &card.core.id;
    let card_name = &card.core.name;

    info!(
        scryfall_id = %scryfall_id,
        card_name = %card_name,
        width = layout.width,
        height = layout.height,
        layout = %card.core.layout,
        "Generating card image"
    );

    // Compose renderers in order
    let renderers: Vec<Box<dyn ElementRenderer>> = vec![
        Box::new(CardArtRenderer),
        Box::new(NameRenderer),
        Box::new(ManaCostRenderer),
        Box::new(TypeLineRenderer),
        Box::new(OracleTextRenderer),
        Box::new(ArtistRenderer),
        Box::new(SetIconRenderer),
        Box::new(SetCodeRenderer),
        Box::new(PowerToughnessRenderer),
        Box::new(BorderRenderer),
    ];

    // Execute each renderer
    for renderer in renderers {
        renderer.render(card, &mut card_img, &mut layout).await?;
    }

    info!(
        scryfall_id = %scryfall_id,
        "Card image generated successfully"
    );

    Ok(card_img)
}

/// Regular card renderer
pub struct RegularCardRenderer<'a> {
    pub card: &'a OracleScryfallCard,
}

impl<'a> CardRenderer for RegularCardRenderer<'a> {
    async fn render(&self, layout: &Layout) -> Result<RgbImage, Box<dyn std::error::Error>> {
        render_card_face(self.card, layout).await
    }
}

/// MDFC card renderer (renders side-by-side)
pub struct MDFCCardRenderer<'a> {
    pub front: &'a OracleScryfallCard,
    pub back: &'a OracleScryfallCard,
}

impl<'a> CardRenderer for MDFCCardRenderer<'a> {
    async fn render(&self, layout: &Layout) -> Result<RgbImage, Box<dyn std::error::Error>> {
        let front_img = render_card_face(self.front, layout).await?;
        let back_img = render_card_face(self.back, layout).await?;

        let mut composed = RgbImage::new(front_img.width() * 2, front_img.height());
        imageops::overlay(&mut composed, &front_img, 0, 0);
        imageops::overlay(&mut composed, &back_img, front_img.width() as i64, 0);

        Ok(composed)
    }
}

/// Main card print handler
pub struct CardPrint<'a> {
    card_type: &'a CardType,
}

impl<'a> CardPrint<'a> {
    pub fn new(card_type: &'a CardType) -> Self {
        Self { card_type }
    }

    pub async fn render(&self, out_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Load fonts once
        let serif_font_path =
            "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/fonts/Mplantin.ttf";
        let sanserif_font_path =
            "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/fonts/tahoma.ttf";

        let (serif_font, sanserif_font) = Layout::load_fonts(serif_font_path, sanserif_font_path)?;

        let layout = Layout {
            serif_font,
            sanserif_font,
            ..Layout::default()
        };

        let image = match self.card_type {
            CardType::Regular(card) => RegularCardRenderer { card }.render(&layout).await?,
            CardType::MDFC { front, back } => {
                MDFCCardRenderer { front, back }.render(&layout).await?
            }
        };

        image.save(out_path)?;
        Ok(())
    }
}
