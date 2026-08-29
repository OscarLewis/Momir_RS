use crate::{
    card::{
        card_type::{CardRenderer, CardType},
        element_renderers::{
            ArtistRenderer, BorderRenderer, CardArtRenderer, ElementRenderer, ManaCostRenderer,
            NameRenderer, OracleAdventureTextRenderer, OracleTextRenderer, PowerToughnessRenderer,
            SetCodeRenderer, SetIconRenderer, TypeLineRenderer,
        },
    },
    layout::Layout,
};
use image::{Rgb, RgbImage, imageops};
use scryfall_oracle::{CardFace, OracleScryfallCard};
use std::path::PathBuf;
use tracing::{debug, info};

/// Handles rendering a single card face
async fn render_card_face(
    card: &OracleScryfallCard,
    face: Option<&CardFace>,
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
        renderer
            .render(card, face, &mut card_img, &mut layout)
            .await?;
    }

    info!(
        scryfall_id = %scryfall_id,
        "Card image generated successfully"
    );

    Ok(card_img)
}

/// Handles rendering a single card face
async fn render_adventure_card_face(
    card: &OracleScryfallCard,
    main_face: Option<&CardFace>,
    adventure_face: Option<&CardFace>,
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

    enum RenderFace {
        Main,
        Adventure,
        None,
    }

    let renderers: Vec<(RenderFace, Box<dyn ElementRenderer>)> = vec![
        (RenderFace::None, Box::new(CardArtRenderer)),
        (RenderFace::Main, Box::new(NameRenderer)),
        (RenderFace::Main, Box::new(ManaCostRenderer)),
        (RenderFace::Main, Box::new(TypeLineRenderer)),
        (RenderFace::Main, Box::new(OracleAdventureTextRenderer)),
        (RenderFace::None, Box::new(ArtistRenderer)),
        (RenderFace::None, Box::new(SetIconRenderer)),
        (RenderFace::None, Box::new(SetCodeRenderer)),
        (RenderFace::None, Box::new(PowerToughnessRenderer)),
        (RenderFace::None, Box::new(BorderRenderer)),
    ];

    for (render_face, renderer) in renderers {
        let face = match render_face {
            RenderFace::Main => main_face,
            RenderFace::Adventure => adventure_face,
            RenderFace::None => None,
        };

        renderer
            .render(card, face, &mut card_img, &mut layout)
            .await?;
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
        render_card_face(self.card, None, layout).await
    }
}

/// MDFC card renderer (renders side-by-side)
pub struct MDFCCardRenderer<'a> {
    pub card: &'a OracleScryfallCard,
}

impl<'a> CardRenderer for MDFCCardRenderer<'a> {
    async fn render(&self, layout: &Layout) -> Result<RgbImage, Box<dyn std::error::Error>> {
        let faces = self.card.core.card_faces.as_ref().ok_or("No card faces")?;
        // debug!(faces = ?faces, "MDFC Card Faces");

        if faces.len() < 2 {
            return Err("Expected at least 2 faces".into());
        }
        if faces.len() > 2 {
            return Err("Expected only 2 faces".into());
        }
        let front_img = render_card_face(self.card, Some(&faces[0]), layout).await?;
        let back_img = render_card_face(self.card, Some(&faces[1]), layout).await?;

        // Composite side-by-side with 20 px white buffer
        let buffer = 20;

        let mut composed = RgbImage::from_pixel(
            front_img.width() * 2 + buffer,
            front_img.height(),
            Rgb([255, 255, 255]),
        );

        imageops::overlay(&mut composed, &front_img, 0, 0);
        imageops::overlay(
            &mut composed,
            &back_img,
            (front_img.width() + buffer) as i64,
            0,
        );

        Ok(composed)
    }
}

/// Regular card renderer
pub struct AdventureCardRenderer<'a> {
    pub card: &'a OracleScryfallCard,
}

impl<'a> CardRenderer for AdventureCardRenderer<'a> {
    async fn render(&self, layout: &Layout) -> Result<RgbImage, Box<dyn std::error::Error>> {
        let faces = self.card.core.card_faces.as_ref().ok_or("No card faces")?;
        // debug!(faces = ?faces, "MDFC Card Faces");

        if faces.len() < 2 {
            return Err("Expected at least 2 faces".into());
        }
        if faces.len() > 2 {
            return Err("Expected only 2 faces".into());
        }
        render_adventure_card_face(self.card, Some(&faces[0]), Some(&faces[1]), layout).await
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
            CardType::MDFC(card) => MDFCCardRenderer { card }.render(&layout).await?,
            CardType::Adventure(card) => AdventureCardRenderer { card }.render(&layout).await?,
            CardType::Omen(card) => AdventureCardRenderer { card }.render(&layout).await?,
            CardType::Prepare(card) => AdventureCardRenderer { card }.render(&layout).await?,
        };

        image.save(out_path)?;
        Ok(())
    }
}
