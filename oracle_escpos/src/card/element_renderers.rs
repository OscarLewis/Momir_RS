use crate::{
    art::CardArtPipeline,
    layout::{BorderStyle, Layout},
    render::{draw_border, draw_svg, draw_text, draw_text_around_border, text_width},
};
use async_trait::async_trait;
use image::{Rgb, RgbImage, imageops};
use scryfall_oracle::{OracleScryfallCard, ScryfallClient, sets::sets::ScryfallSet};
use tracing::{debug, info};

#[async_trait]
/// Trait for individual element renderers
pub trait ElementRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>>;
}

/// Renders card art
pub struct CardArtRenderer;

#[async_trait]
impl ElementRenderer for CardArtRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let card_art = if let Some(image_uris) = &card.print.image_uris {
            let client = ScryfallClient::new()?;
            Some(image_uris.fetch_art_crop(&client).await?.to_vec())
        } else {
            None
        };

        if let Some(card_art) = card_art {
            let card_art_img = image::load_from_memory(&card_art)?;
            let ca_width = card_art_img.width().clone();
            let ca_height = card_art_img.height().clone();

            let art =
                CardArtPipeline::process(card_art_img, layout.art.max_width, layout.art.max_height);

            let scale = (layout.art.max_width as f64 / ca_width as f64)
                .min(layout.art.max_height as f64 / ca_height as f64)
                .min(1.0);

            let render_width = (ca_width as f64 * scale) as i64;
            let draw_x = (layout.art.x as i64) + ((layout.art.max_width as i64) - render_width) / 2;

            imageops::overlay(canvas, &art, draw_x, layout.art.y);
        }

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
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let is_funny = card.core.set_type == "funny";

        let border_style = {
            let name_style = &layout.name;
            let name_font_data = layout.font_data(name_style.font);
            let name_width = layout.text_width(&card.core.name, name_style);

            if let Some(long_text_font_size) = name_style.long_text_font_size {
                if name_width > name_style.wrap_width as f32 {
                    let total_text_len =
                        text_width(&card.core.name, name_font_data, long_text_font_size, 0.5);
                    let standard_wrap_limit = (name_style.wrap_width + 10) as f32;

                    if total_text_len > layout.border_path.bottom_threshold() && is_funny {
                        BorderStyle::FullWrap
                    } else if total_text_len > standard_wrap_limit && is_funny {
                        BorderStyle::SemiWrap
                    } else {
                        BorderStyle::LongName
                    }
                } else {
                    BorderStyle::Standard
                }
            } else {
                BorderStyle::Standard
            }
        };

        layout.border_style = border_style;
        border_style.apply_layout_adjustments(layout);

        let name_style = &layout.name;
        let name_font_data = layout.font_data(name_style.font);

        match layout.border_style {
            BorderStyle::FullWrap | BorderStyle::SemiWrap => {
                draw_text_around_border(
                    canvas,
                    &card.core.name,
                    name_font_data,
                    name_style.font_size,
                    name_style.letter_spacing,
                    &layout.border_path,
                );
            }
            BorderStyle::Standard | BorderStyle::LongName => {
                draw_text(
                    canvas,
                    &card.core.name,
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

/// Renders mana cost
pub struct ManaCostRenderer;
#[async_trait]
impl ElementRenderer for ManaCostRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mana_cost_style = &layout.cost;
        let mana_cost_font_data = layout.font_data(mana_cost_style.font);
        if let Some(mana_cost) = &card.core.mana_cost {
            let cost_width = layout.text_width(&mana_cost, &layout.cost);

            let (cost_font_size, cost_x) = if cost_width > layout.cost.wrap_width as f32 {
                let long_cost_width = crate::render::text_width(
                    &mana_cost,
                    mana_cost_font_data,
                    layout.font_sizes.long_cost,
                    mana_cost_style.letter_spacing,
                );

                let adjusted_x = (layout.width as i32 - long_cost_width as i32)
                    - mana_cost_style.margin_right as i32;
                debug!(
                    scenario = "long_cost",
                    long_cost_width = long_cost_width,
                    adjusted_x = adjusted_x,
                    "Using long cost sizing"
                );
                (layout.font_sizes.long_cost, adjusted_x)
            } else {
                let normal_x =
                    (layout.width as i32 - cost_width as i32) - mana_cost_style.margin_right as i32;
                debug!(
                    scenario = "normal_cost",
                    cost_width = cost_width,
                    normal_x = normal_x,
                    "Using normal cost sizing"
                );
                (mana_cost_style.font_size, normal_x)
            };

            draw_text(
                canvas,
                &mana_cost,
                cost_x,
                mana_cost_style.y,
                mana_cost_font_data,
                cost_font_size,
                mana_cost_style.letter_spacing,
                mana_cost_style.wrap_width,
            );
        }

        Ok(())
    }
}

/// Renders type line
pub struct TypeLineRenderer;
#[async_trait]
impl ElementRenderer for TypeLineRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let type_line = card.core.type_line.clone().unwrap_or_default();
        let type_style = &layout.type_line;
        let type_font_data = layout.font_data(type_style.font);

        debug!(font_size = type_style.font_size, "Rendering type line");

        let type_line_end_y = draw_text(
            canvas,
            &type_line,
            type_style.x,
            type_style.y,
            type_font_data,
            type_style.font_size,
            type_style.letter_spacing,
            type_style.wrap_width,
        );

        // Store for oracle text renderer to use
        layout.type_line_end_y = type_line_end_y;

        Ok(())
    }
}

/// Renders oracle text
pub struct OracleTextRenderer;
#[async_trait]
impl ElementRenderer for OracleTextRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let oracle_text = card.core.oracle_text.clone().unwrap_or_default();
        let rules_style = &layout.rules;
        let rules_font_data = layout.font_data(rules_style.font);
        let rules_y = rules_style.y.max(layout.type_line_end_y);

        debug!(
            font_size = rules_style.font_size,
            oracle_text_length = oracle_text.len(),
            "Rendering oracle text"
        );

        draw_text(
            canvas,
            &oracle_text,
            rules_style.x,
            rules_y,
            rules_font_data,
            rules_style.font_size,
            rules_style.letter_spacing,
            rules_style.wrap_width,
        );

        Ok(())
    }
}

/// Renders artist credit
pub struct ArtistRenderer;
#[async_trait]
impl ElementRenderer for ArtistRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(artist_name) = &card.print.artist {
            let artist_style = &layout.artist;
            let artist_font_data = layout.font_data(artist_style.font);

            draw_text(
                canvas,
                &format!("Art by {}", &artist_name),
                artist_style.x,
                artist_style.y,
                artist_font_data,
                artist_style.font_size,
                artist_style.letter_spacing,
                artist_style.wrap_width,
            );
        }

        Ok(())
    }
}

/// Renders set icon
pub struct SetIconRenderer;
#[async_trait]
impl ElementRenderer for SetIconRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if card.core.set_icon_svg_uri.is_some() {
            debug!("Loading set icon from Scryfall");

            let client = ScryfallClient::new()?;
            let set = ScryfallSet::from_id(&card.core.set_id, &client).await?;
            let svg_data = set.get_svg_bytes(&client).await?;

            let set_icon = &layout.set_icon;

            draw_svg(
                canvas,
                &svg_data,
                set_icon.x,
                set_icon.y,
                set_icon.width,
                set_icon.height,
            )?;
        }

        Ok(())
    }
}

/// Renders set code
pub struct SetCodeRenderer;
#[async_trait]
impl ElementRenderer for SetCodeRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let metadata_style = &layout.set_code;
        let metadata_font_data = layout.font_data(metadata_style.font);

        draw_text(
            canvas,
            &format!(
                "{} {}",
                card.core.set.to_uppercase(),
                card.print.collector_number
            ),
            metadata_style.x,
            metadata_style.y,
            metadata_font_data,
            metadata_style.font_size,
            metadata_style.letter_spacing,
            metadata_style.wrap_width,
        );

        Ok(())
    }
}

/// Renders power/toughness
pub struct PowerToughnessRenderer;
#[async_trait]
impl ElementRenderer for PowerToughnessRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pow_tough_style = &layout.pow_tough_style;
        let pow_tough_font_data = layout.font_data(pow_tough_style.font);

        if let (Some(power), Some(toughness)) = (&card.core.power, &card.core.toughness) {
            debug!(
                power = %power,
                toughness = %toughness,
                "Rendering power and toughness"
            );

            draw_text(
                canvas,
                &format!("{power}/{toughness}"),
                pow_tough_style.x,
                pow_tough_style.y,
                pow_tough_font_data,
                pow_tough_style.font_size,
                pow_tough_style.letter_spacing,
                pow_tough_style.wrap_width,
            );
        } else {
            debug!("Card has no power and toughness");
        }

        Ok(())
    }
}

/// Renders border
pub struct BorderRenderer;
#[async_trait]
impl ElementRenderer for BorderRenderer {
    async fn render(
        &self,
        _card: &OracleScryfallCard,
        canvas: &mut RgbImage,
        _layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Rendering card border");
        draw_border(canvas);
        Ok(())
    }
}
