use crate::render::{draw_border, draw_svg, draw_text, text_width};
use image::{Rgb, RgbImage, imageops};
use scryfall_oracle::{OracleScryfallCard, ScryfallClient, sets::sets::ScryfallSet};
use std::path::PathBuf;
use tracing::{debug, info};

/// Represents a rendered Magic card image
pub struct CardImage {
    /// The Scryfall ID of the card
    pub scryfall_id: String,

    /// The card data used to generate the image
    card: OracleScryfallCard,
}

impl CardImage {
    /// Creates a new card image from an Oracle Scryfall card
    pub fn new(card: OracleScryfallCard) -> Self {
        debug!(
            scryfall_id = %card.core.id,
            card_name = %card.core.name,
            "Creating CardImage"
        );

        Self {
            scryfall_id: card.core.id.clone(),
            card,
        }
    }

    /// Generates the card image and saves it as a PNG
    pub async fn generate(self, out_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        const DEFAULT_LINE_WRAP_WIDTH: i32 = 372;
        const MAX_ART_WIDTH: u32 = 300;
        const MAX_ART_HEIGHT: u32 = 200;
        const CARD_ART_HEIGHT: i64 = 80;
        const WIDTH: u32 = 412;
        const HEIGHT: u32 = 576;
        const RULES_TEXT_Y_POS: i32 = 340;
        const FONT_NAME: f32 = 34.0;
        const FONT_LONG_NAME: f32 = 24.0;
        const FONT_TYPE_LINE: f32 = 22.0;
        const FONT_RULES: f32 = 19.0;
        const FONT_FLAVOR: f32 = 17.0;
        const NAME_Y_POS: i32 = 35;
        const FONT_METADATA: f32 = 14.0;

        info!(
            scryfall_id = %self.scryfall_id,
            card_name = %self.card.core.name,
            width = WIDTH,
            height = HEIGHT,
            "Generating card image"
        );

        let mut card_img = RgbImage::from_pixel(WIDTH, HEIGHT, Rgb([255, 255, 255]));

        let oracle_text = self.card.core.oracle_text.unwrap_or_default();
        let type_line = self.card.core.type_line.unwrap_or_default();

        let card_art = if let Some(image_uris) = &self.card.print.image_uris {
            let client = ScryfallClient::new()?;
            Some(image_uris.fetch_art_crop(&client).await?.to_vec())
        } else {
            None
        };

        if let Some(card_art) = card_art {
            let card_art_img = image::load_from_memory(&card_art)?;

            let art = card_art_img
                .resize(
                    MAX_ART_WIDTH,
                    MAX_ART_HEIGHT,
                    image::imageops::FilterType::Lanczos3,
                )
                .grayscale()
                .to_rgb8();

            imageops::overlay(&mut card_img, &art, 60, CARD_ART_HEIGHT);
        }

        debug!(
            oracle_text_length = oracle_text.len(),
            "Loaded card oracle text"
        );

        let font_path =
            "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/fonts/Mplantin.ttf";

        debug!(path = font_path, "Loading font");

        let font_data = std::fs::read(font_path)?;
        debug!(font_size = FONT_NAME, "Rendering card name");

        // Check for a comma in a name, might be a good place to split?
        // if !&self.card.core.name.contains(',') {}

        const NAME_MAX_WIDTH_PRE_ADJUST: f32 = 372.0;
        const NAME_MAX_WIDTH_POST_ADJUST: f32 = 377.1;

        let name_width = text_width(&self.card.core.name, &font_data, FONT_NAME, 0.0);
        let long_name = name_width > NAME_MAX_WIDTH_PRE_ADJUST;

        let font_name_size = if long_name { FONT_LONG_NAME } else { FONT_NAME };
        let font_name_spacing = if long_name { 0.5 } else { 0.0 };

        if long_name {
            let post_width = text_width(
                &self.card.core.name,
                &font_data,
                font_name_size,
                font_name_spacing,
            );
            info!(
                initial_name_width = name_width,
                post_name_width = post_width,
                "Long name found"
            );
            if post_width > NAME_MAX_WIDTH_POST_ADJUST {
                info!(post_name_width = post_width, "Really long name found");
            }
        }

        // TODO Draw Option<card.print.image_uris> art_crop here

        draw_text(
            &mut card_img,
            &self.card.core.name,
            20,
            NAME_Y_POS,
            &font_data,
            font_name_size,
            font_name_spacing,
            DEFAULT_LINE_WRAP_WIDTH,
        );

        debug!(
            font_size = FONT_RULES,
            oracle_text_length = oracle_text.len(),
            "Rendering oracle text"
        );

        draw_text(
            &mut card_img,
            &type_line,
            20,
            RULES_TEXT_Y_POS - 30,
            &font_data,
            FONT_RULES,
            0.0,
            DEFAULT_LINE_WRAP_WIDTH,
        );

        draw_text(
            &mut card_img,
            &oracle_text,
            20,
            RULES_TEXT_Y_POS,
            &font_data,
            FONT_RULES,
            1.0,
            DEFAULT_LINE_WRAP_WIDTH,
        );

        if let Some(svg_uri) = &self.card.core.set_icon_svg_uri {
            debug!(uri = %svg_uri, "Loading set icon from Scryfall");

            let client = ScryfallClient::new()?;

            let set = ScryfallSet::from_id(&self.card.core.set_id, &client).await?;

            let svg_data = set.get_svg_bytes(&client).await?;

            draw_svg(&mut card_img, &svg_data, 20, HEIGHT - 80, 50, 50)?;
        }

        draw_text(
            &mut card_img,
            &self.card.core.set.to_uppercase(),
            20,
            (HEIGHT - 10).try_into().unwrap(),
            &font_data,
            FONT_RULES,
            1.0,
            DEFAULT_LINE_WRAP_WIDTH,
        );

        if let (Some(power), Some(toughness)) = (&self.card.core.power, &self.card.core.toughness) {
            debug!(
                power = %power,
                toughness = %toughness,
                "Rendering power and toughness"
            );

            draw_text(
                &mut card_img,
                &format!("{power}/{toughness}"),
                (WIDTH - 70).try_into().unwrap(),
                (HEIGHT - 20) as i32,
                &font_data,
                FONT_NAME,
                0.0,
                DEFAULT_LINE_WRAP_WIDTH,
            );
        } else {
            debug!("Card has no power and toughness");
        }

        debug!("Rendering card border");

        draw_border(&mut card_img);

        card_img.save(out_path)?;

        info!(
            scryfall_id = %self.scryfall_id,
            "Card image generated successfully"
        );

        Ok(())
    }
}
