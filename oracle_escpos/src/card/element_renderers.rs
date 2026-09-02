use crate::{
    art::CardArtPipeline,
    card::svg::LOYALTY_SHIELD_SVG,
    layout::{Layout, NameStyle, WrapStyle},
    render::{
        draw_border, draw_svg, draw_text, draw_text_around_border, draw_text_rotated_270,
        draw_vertical_line, text_width, wrapped_line_count, wrapped_text_height,
    },
};
use async_trait::async_trait;
use image::{Rgb, RgbImage, imageops};
use scryfall_oracle::{CardFace, OracleScryfallCard, ScryfallClient, sets::sets::ScryfallSet};
use swash::FontRef;
use tracing::{debug, info};

const SCRYFALL_USER_AGENT: &str = "oracle_escpos/1.0";

#[async_trait]
pub trait ElementRenderer: Send + Sync {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
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
            Some(image_uris.fetch_art(&client).await?.to_vec())
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

/// Renders mana cost
pub struct ManaCostRenderer;
#[async_trait]
impl ElementRenderer for ManaCostRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mana_cost_style = &layout.cost;
        let mana_cost_font_data = layout.font_data(mana_cost_style.font);
        //     let image_uris = face
        // .and_then(|f| f.image_uris.as_ref())
        // .or_else(|| card.print.image_uris.as_ref());
        let mana_cost = face
            .and_then(|f| f.mana_cost.as_ref())
            .or_else(|| card.core.mana_cost.as_ref());

        if let Some(mana_cost) = mana_cost {
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
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let type_line = face
            .and_then(|f| f.type_line.as_ref())
            .or_else(|| card.core.type_line.as_ref())
            .cloned()
            .unwrap_or_default();
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

        // First attempt with default font size
        let mut font_size = rules_style.font_size;
        let mut total_height = layout.wrapped_text_height(&oracle_text, rules_style);

        // Fall back to smaller font size if max_length is exceeded
        if let Some(max_length) = rules_style.max_length {
            if total_height > max_length {
                if let Some(long_font_size) = rules_style.long_text_font_size {
                    debug!(
                        initial_height = total_height,
                        max_length,
                        long_font_size,
                        "Oracle text exceeds max_length; falling back to long_text_font_size"
                    );

                    font_size = long_font_size;

                    // Recalculate text height with reduced font size
                    let mut scaled_style = rules_style.clone();
                    scaled_style.font_size = long_font_size;
                    total_height = layout.wrapped_text_height(&oracle_text, &scaled_style);
                }

                if total_height > max_length {
                    debug!(
                        total_height,
                        max_length,
                        overflow = total_height - max_length,
                        font_size,
                        "Oracle text height still exceeds max_length boundaries"
                    );
                }
            }
        }

        debug!(
            font_size,
            oracle_text_length = oracle_text.len(),
            total_height,
            "Rendering oracle text"
        );

        draw_text(
            canvas,
            &oracle_text,
            rules_style.x,
            rules_y,
            rules_font_data,
            font_size,
            rules_style.letter_spacing,
            rules_style.wrap_width,
        );

        Ok(())
    }
}

/// Renders Adventure oracle text (both Normal and Adventure)
pub struct OracleAdventureTextRenderer;
#[async_trait]
impl ElementRenderer for OracleAdventureTextRenderer {
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

        let adventure_type_text = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(1))
            .and_then(|face| face.type_line.as_deref())
            .unwrap_or_default();

        let adventure_text = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(1))
            .and_then(|face| face.oracle_text.as_deref())
            .unwrap_or_default();

        let adventure_name = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(1))
            .map(|face| face.name.as_str())
            .unwrap_or_default();

        let adventure_name_style = &layout.adventure_name;
        let adventure_name_font_data = layout.font_data(adventure_name_style.font);

        let main_face_oracle_style = &layout.adventure_oracle_text_main_face;
        let main_face_font_data = layout.font_data(main_face_oracle_style.font);
        let main_face_font = FontRef::from_index(main_face_font_data, 0).expect("invalid font");
        let main_face_line_count = wrapped_line_count(
            &adventure_text,
            main_face_font,
            main_face_oracle_style.font_size,
            main_face_oracle_style.letter_spacing,
            main_face_oracle_style.wrap_width,
        );

        let alt_face_font_size = if main_face_line_count > 5 {
            main_face_oracle_style
                .long_text_font_size
                .unwrap_or(main_face_oracle_style.font_size)
        } else {
            main_face_oracle_style.font_size
        };

        let adventure_type_line_style = &layout.adventure_type_line;
        let adventure_type_font_data = layout.font_data(adventure_type_line_style.font);

        let alt_face_oracle_style = &layout.adventure_oracle_text_alt_face;
        let alt_face_font_data = layout.font_data(alt_face_oracle_style.font);

        let rules_y = main_face_oracle_style.y.max(layout.type_line_end_y);

        draw_vertical_line(canvas, (layout.width / 2) as i32, rules_y, 180, 2);

        draw_text(
            canvas,
            &oracle_text,
            main_face_oracle_style.x,
            rules_y,
            main_face_font_data,
            main_face_oracle_style.font_size,
            main_face_oracle_style.letter_spacing,
            main_face_oracle_style.wrap_width,
        );

        let sub_type_y = draw_text(
            canvas,
            adventure_type_text,
            adventure_type_line_style.x,
            rules_y,
            adventure_type_font_data,
            adventure_type_line_style.font_size,
            adventure_type_line_style.letter_spacing,
            adventure_type_line_style.wrap_width,
        );

        let name_y = draw_text(
            canvas,
            &adventure_name,
            adventure_name_style.x,
            sub_type_y,
            adventure_name_font_data,
            adventure_name_style.font_size,
            adventure_name_style.letter_spacing,
            adventure_name_style.wrap_width,
        );

        let mana_cost = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(1))
            .and_then(|face| face.mana_cost.as_deref());

        let mana_cost_style = &layout.adventure_mana_cost;
        let mana_cost_font_data = layout.font_data(mana_cost_style.font);
        let mut cmc_y = 0;
        if let Some(mana_cost) = mana_cost {
            let cost_width = layout.text_width(mana_cost, mana_cost_style);

            let (cost_font_size, cost_x) = if cost_width > mana_cost_style.wrap_width as f32 {
                debug!(scenario = "long_cost", cost_width, "Using long cost sizing");

                (
                    layout.font_sizes.long_cost,
                    mana_cost_style.margin_left as i32,
                )
            } else {
                debug!(
                    scenario = "normal_cost",
                    cost_width, "Using normal cost sizing"
                );

                (
                    mana_cost_style.font_size,
                    mana_cost_style.margin_left as i32,
                )
            };

            cmc_y = draw_text(
                canvas,
                mana_cost,
                cost_x,
                name_y,
                mana_cost_font_data,
                cost_font_size,
                mana_cost_style.letter_spacing,
                mana_cost_style.wrap_width,
            );
        }

        draw_text(
            canvas,
            adventure_text,
            alt_face_oracle_style.x,
            alt_face_oracle_style.y.max(cmc_y),
            alt_face_font_data,
            alt_face_font_size,
            alt_face_oracle_style.letter_spacing,
            alt_face_oracle_style.wrap_width,
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
        _face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(artist_name) = &card.print.artist {
            let artist_style = &layout.artist;
            let text = format!("Art by {}", artist_name);
            let artist_font_data = layout.font_data(artist_style.font);

            let name_width = layout.text_width(&text, artist_style);
            let font_size = if name_width > artist_style.wrap_width as f32 {
                artist_style
                    .long_text_font_size
                    .unwrap_or(artist_style.font_size)
            } else {
                artist_style.font_size
            };

            // Calculate total height using the helper
            let total_height = wrapped_text_height(
                &text,
                artist_font_data,
                font_size,
                artist_style.letter_spacing,
                artist_style.wrap_width,
            );

            // Compute extra height above single-line baseline
            let line_height = (font_size * 1.25).round() as i32;
            let extra_height = total_height.saturating_sub(line_height);

            let y = artist_style.y - extra_height;

            draw_text(
                canvas,
                &text,
                artist_style.x,
                y,
                artist_font_data,
                font_size,
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
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if card.core.set_icon_svg_uri.is_some() {
            debug!("Loading set icon from Scryfall");

            let client = ScryfallClient::new(Some(SCRYFALL_USER_AGENT))?;
            let set = ScryfallSet::from_id(&card.core.set_id, &client).await?;
            let svg_data = set.get_svg_bytes(&client).await?;

            let set_icon = &layout.set_icon;

            draw_svg(
                canvas,
                &svg_data,
                set_icon.x,
                set_icon.y,
                set_icon.max_width,
                set_icon.max_height,
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
        face: Option<&CardFace>,
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
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let pow_tough_style = &layout.pow_tough_style;
        let pow_tough_font_data = layout.font_data(pow_tough_style.font);

        let power = face
            .and_then(|f| f.power.as_ref())
            .or_else(|| card.core.power.as_ref());

        let toughness = face
            .and_then(|f| f.toughness.as_ref())
            .or_else(|| card.core.toughness.as_ref());

        if let (Some(power), Some(toughness)) = (power, toughness) {
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

/// Renders power/toughness
pub struct PlaneswalkerLoyaltyRenderer;
#[async_trait]
impl ElementRenderer for PlaneswalkerLoyaltyRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let loyalty_style = &layout.planeswalker_loyalty_style;
        let loyalty_font_data = layout.font_data(loyalty_style.font);

        if let Some(loyalty) = card.core.loyalty.as_ref() {
            debug!(loyalty = loyalty, "Rendering Planeswalker loyalty");

            draw_text(
                canvas,
                loyalty,
                loyalty_style.x,
                loyalty_style.y,
                loyalty_font_data,
                loyalty_style.font_size,
                loyalty_style.letter_spacing,
                loyalty_style.wrap_width,
            );
        } else {
            debug!("Card has no loyalty");
        }

        Ok(())
    }
}

/// Renders set icon
pub struct PlaneswalkerShieldRenderer;
#[async_trait]
impl ElementRenderer for PlaneswalkerShieldRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let shield_style = &layout.planeswalker_loyalty_shield;
        debug!("Rendering Planwalker loyalty shield");
        draw_svg(
            canvas,
            LOYALTY_SHIELD_SVG,
            shield_style.x,
            shield_style.y,
            shield_style.max_width,
            shield_style.max_height,
        )?;

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
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        _layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        debug!("Rendering card border");
        draw_border(canvas);
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
