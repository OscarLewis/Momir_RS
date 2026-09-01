use crate::{
    art::CardArtPipeline,
    card::element_renderers::ElementRenderer,
    layout::{Layout, NameStyle, WrapStyle},
    render::{draw_text, draw_text_around_border, draw_text_rotated_270, text_width},
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
        let rules_style = &layout.rules;
        let rules_font_data = layout.font_data(rules_style.font);
        let rules_y = rules_style.y.max(layout.type_line_end_y);

        todo!();

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

/// Renders card name with border wrapping logic
pub struct NameRenderer;

#[async_trait]
impl ElementRenderer for NameRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let is_funny = card.core.set_type == "funny";
        let name = face.map(|f| &f.name).unwrap_or_else(|| &card.core.name);
        let (name_style, wrap_style) = {
            let name_style = &layout.name;
            let name_font_data = layout.font_data(name_style.font);
            let name_width = layout.text_width(name, name_style);

            if let Some(long_text_font_size) = name_style.long_text_font_size {
                if name_width > name_style.wrap_width as f32 {
                    let total_text_len = text_width(name, name_font_data, long_text_font_size, 0.5);
                    let standard_wrap_limit = (name_style.wrap_width + 10) as f32;

                    if total_text_len > layout.border_path.bottom_threshold() && is_funny {
                        (NameStyle::LongName, WrapStyle::FullWrap)
                    } else if total_text_len > standard_wrap_limit && is_funny {
                        (NameStyle::LongName, WrapStyle::SemiWrap)
                    } else {
                        (NameStyle::LongName, WrapStyle::Standard)
                    }
                } else {
                    (NameStyle::Standard, WrapStyle::Standard)
                }
            } else {
                (NameStyle::Standard, WrapStyle::Standard)
            }
        };

        name_style.apply_layout_adjustments(layout);
        wrap_style.apply_layout_adjustments(layout);

        let name_style = &layout.name;
        let name_font_data = layout.font_data(name_style.font);

        match wrap_style {
            WrapStyle::FullWrap | WrapStyle::SemiWrap => {
                draw_text_around_border(
                    canvas,
                    name,
                    name_font_data,
                    name_style.font_size,
                    name_style.letter_spacing,
                    &layout.border_path,
                );
            }
            WrapStyle::Standard => {
                draw_text(
                    canvas,
                    name,
                    name_style.x,
                    name_style.y,
                    name_font_data,
                    name_style.font_size,
                    name_style.letter_spacing,
                    name_style.wrap_width,
                );
            }
        }

        Ok(())
    }
}
