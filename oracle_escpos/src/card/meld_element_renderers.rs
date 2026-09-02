use crate::{
    art::CardArtPipeline,
    card::{element_renderers::ElementRenderer, svg::LOYALTY_SHIELD_SVG},
    layout::{Layout, NameStyle, WrapStyle},
    render::{
        draw_svg_rotated_270, draw_text, draw_text_around_border, draw_text_rotated_270, text_width,
    },
};
use async_trait::async_trait;
use image::{RgbImage, imageops};
use scryfall_oracle::{CardFace, OracleScryfallCard, ScryfallClient};
use tracing::{debug, info};

const SCRYFALL_USER_AGENT: &str = "oracle_escpos/1.0";

/// Renders meld back card art
pub struct MeldBackCardArtRenderer;
#[async_trait]
impl ElementRenderer for MeldBackCardArtRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let image_uris = face
            .and_then(|f| f.image_uris.as_ref())
            .or_else(|| card.print.image_uris.as_ref());

        debug!(
            has_face = face.is_some(),
            has_image_uris = image_uris.is_some(),
            "CardArtRenderer state"
        );

        let card_art = if let Some(image_uris) = image_uris {
            let client = ScryfallClient::new(Some(SCRYFALL_USER_AGENT))?;
            Some(image_uris.fetch_display(&client).await?.to_vec())
        } else {
            None
        };

        if let Some(card_art) = card_art {
            let card_art_img = image::load_from_memory(&card_art)?;

            let half_height = card_art_img.height() / 2;

            let card_art_img = card_art_img
                .crop_imm(0, 0, card_art_img.width(), half_height)
                .crop_imm(20, 88, card_art_img.width() - 40, half_height - 88)
                .rotate270();

            let ca_width = card_art_img.width();
            let ca_height = card_art_img.height();

            card_art_img.save("/tmp/meld_card_back_art.png")?;

            let art = CardArtPipeline::process(
                card_art_img,
                layout.meld_card_back_art.max_width,
                layout.meld_card_back_art.max_height,
            );
            let scale = (layout.meld_card_back_art.max_width as f64 / art.width() as f64)
                .min(layout.meld_card_back_art.max_height as f64 / art.height() as f64)
                .min(1.0);

            let render_width = (art.width() as f64 * scale) as u32;
            let render_height = (art.height() as f64 * scale) as u32;

            let art = imageops::resize(
                &art,
                render_width,
                render_height,
                imageops::FilterType::Lanczos3,
            );
            let margin_right = layout.meld_card_back_art.margin_right.unwrap_or(0);

            let margin_right = layout.meld_card_back_art.margin_right.unwrap_or(0);

            let x = canvas.width() as i64 - margin_right - art.width() as i64;
            let y = (canvas.height() as i64 - art.height() as i64) / 2;

            imageops::overlay(canvas, &art, x, y);
        }

        Ok(())
    }
}

/// Renders oracle text
pub struct MeldOracleTextRenderer;
#[async_trait]
impl ElementRenderer for MeldOracleTextRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let oracle_text = face
            .and_then(|f| f.oracle_text.as_ref())
            .or_else(|| card.core.oracle_text.as_ref())
            .cloned()
            .unwrap_or_default();
        let oracle_style = &layout.meld_oracle;
        let oracle_font_data = layout.font_data(oracle_style.font);
        let rules_x = oracle_style.x.max(layout.meld_type_line_end_x);

        let oracle_width = layout.wrapped_text_width(&oracle_text, oracle_style);

        // Center vertically
        // Center vertically
        let center_y = canvas.height() as f32 / 2.0;
        let baseline_y = (center_y + (oracle_width / 2.0).round()) as i32;

        // let font_size = match name_style.long_text_font_size {
        //     Some(long_size) if name_width > name_style.wrap_width as f32 => long_size,
        //     _ => name_style.font_size,
        // };

        debug!(
            font_size = oracle_style.font_size,
            oracle_text_length = oracle_text.len(),
            "Rendering meld oracle text"
        );

        draw_text_rotated_270(
            canvas,
            &oracle_text,
            rules_x,
            // oracle_style.y,
            baseline_y,
            oracle_font_data,
            oracle_style.font_size,
            oracle_style.letter_spacing,
            oracle_style.wrap_width,
        );

        Ok(())
    }
}

pub struct MeldNameRenderer;

#[async_trait]
impl ElementRenderer for MeldNameRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let name = face.map(|f| &f.name).unwrap_or_else(|| &card.core.name);

        let name_style = &layout.meld_name;
        let font_data = layout.font_data(name_style.font);

        let name_width = layout.text_width(name, name_style);

        let font_size = match name_style.long_text_font_size {
            Some(long_size) if name_width > name_style.wrap_width as f32 => long_size,
            _ => name_style.font_size,
        };

        // Center vertically
        let center_y = layout.height as i32 / 2;
        let baseline_y = center_y + (name_width / 2.0).round() as i32;

        draw_text_rotated_270(
            canvas,
            name,
            name_style.x,
            // name_style.y,
            baseline_y,
            font_data,
            font_size,
            name_style.letter_spacing,
            name_style.wrap_width,
        );

        Ok(())
    }
}

/// Renders type line
pub struct MeldTypeLineRenderer;
#[async_trait]
impl ElementRenderer for MeldTypeLineRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let type_line = face
            .and_then(|f| f.type_line.as_ref())
            .or_else(|| card.core.type_line.as_ref())
            .cloned()
            .unwrap_or_default();
        let type_style = &layout.meld_type_line;
        let type_font_data = layout.font_data(type_style.font);

        debug!(font_size = type_style.font_size, "Rendering type line");
        let type_width = layout.text_width(&type_line, type_style);

        // wrapped_text_width

        // Center vertically
        let center_y = layout.height as i32 / 2;
        let baseline_y = center_y + (type_width / 2.0).round() as i32;

        let type_line_end_x = draw_text_rotated_270(
            canvas,
            &type_line,
            type_style.x,
            // type_style.y,
            baseline_y,
            type_font_data,
            type_style.font_size,
            type_style.letter_spacing,
            type_style.wrap_width,
        );

        // Store for oracle text renderer to use
        layout.meld_type_line_end_x = type_line_end_x;

        Ok(())
    }
}

/// Renders set icon
pub struct MeldPlaneswalkerShieldRenderer;
#[async_trait]
impl ElementRenderer for MeldPlaneswalkerShieldRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let meld_shield_style = &layout.meld_planeswalker_loyalty_shield;
        debug!("Rendering Planwalker loyalty shield");
        draw_svg_rotated_270(
            canvas,
            LOYALTY_SHIELD_SVG,
            meld_shield_style.x,
            meld_shield_style.y,
            meld_shield_style.max_width,
            meld_shield_style.max_height,
        )?;

        Ok(())
    }
}

/// Renders set code
pub struct MeldSetCodeRenderer;
#[async_trait]
impl ElementRenderer for MeldSetCodeRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_style = &layout.meld_set_code;
        let metadata_font_data = layout.font_data(metadata_style.font);
        let normal_x = (layout.width - metadata_style.margin_right) as i32;
        draw_text_rotated_270(
            canvas,
            &format!(
                "{} {}",
                card.core.set.to_uppercase(),
                card.print.collector_number
            ),
            // metadata_style.x,
            normal_x,
            metadata_style.y,
            metadata_font_data,
            metadata_style.font_size,
            metadata_style.letter_spacing,
            metadata_style.wrap_width,
        );

        Ok(())
    }
}
