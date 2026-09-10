use crate::{
    art::CardArtPipeline,
    card::element_renderers::ElementRenderer,
    layout::Layout,
    render::{draw_horizontal_line, draw_svg, draw_svg_rotated_270, draw_text_rotated_270},
};
use async_trait::async_trait;
use image::{RgbImage, imageops};
use scryfall_oracle::{CardFace, OracleScryfallCard, ScryfallClient, sets::sets::ScryfallSet};
use tracing::debug;

const SCRYFALL_USER_AGENT: &str = "oracle_escpos/1.0";
pub struct SplitNameRenderer;
#[async_trait]
impl ElementRenderer for SplitNameRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        _face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let first_name = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(0))
            .map(|face| face.name.clone())
            .unwrap_or_else(|| card.core.name.clone());

        let second_name = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(1))
            .map(|face| face.name.clone())
            .unwrap_or_else(|| card.core.name.clone());

        // First Name Setup
        let first_name_style = &layout.split_first_name;
        let first_font_data = layout.font_data(first_name_style.font);

        let first_font_size = match first_name_style.small_text_font_size {
            Some(long_size)
                if layout.text_width(&first_name, first_name_style)
                    > first_name_style.wrap_width as f32 =>
            {
                long_size
            }
            _ => first_name_style.font_size,
        };

        // Second Name Setup
        let second_name_style = &layout.split_second_name;
        let second_font_data = layout.font_data(second_name_style.font);

        let second_font_size = match second_name_style.small_text_font_size {
            Some(long_size)
                if layout.text_width(&second_name, second_name_style)
                    > second_name_style.wrap_width as f32 =>
            {
                long_size
            }
            _ => second_name_style.font_size,
        };

        let first_y: i32 = (layout.height - first_name_style.margin_bottom) as i32;

        let second_y: i32 = ((layout.height / 2) - second_name_style.margin_bottom) as i32;

        // Render Calls
        if !card.is_room() {
            if card.has_keyword("Fuse") {
                draw_horizontal_line(
                    canvas,
                    20,
                    (layout.height / 2) as i32,
                    (layout.width - 110) as i32,
                    2,
                );
            } else {
                draw_horizontal_line(
                    canvas,
                    20,
                    (layout.height / 2) as i32,
                    (layout.width - 40) as i32,
                    2,
                );
            }
        }

        draw_text_rotated_270(
            canvas,
            &first_name,
            first_name_style.x,
            first_y,
            first_font_data,
            first_font_size,
            first_name_style.letter_spacing,
            first_name_style.wrap_width,
        );

        draw_text_rotated_270(
            canvas,
            &second_name,
            second_name_style.x,
            second_y,
            second_font_data,
            second_font_size,
            second_name_style.letter_spacing,
            second_name_style.wrap_width,
        );

        Ok(())
    }
}

pub struct SplitCostRenderer;
#[async_trait]
impl ElementRenderer for SplitCostRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        _face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let first_cost_opt = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(0))
            .map(|face| face.mana_cost.clone())
            .unwrap_or_else(|| card.core.mana_cost.clone());

        let second_cost_opt = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(1))
            .map(|face| face.mana_cost.clone())
            .unwrap_or_else(|| card.core.mana_cost.clone());

        if let Some(first_cost) = first_cost_opt {
            let first_cost_style = &layout.split_first_cost;
            let first_cost_font_data = layout.font_data(first_cost_style.font);

            let first_cost_width = layout.text_width(&first_cost, first_cost_style);
            // First Name Setup

            // let first_cost_font_size = match first_cost_style.small_text_font_size {
            //     Some(long_size) if first_cost_width > (first_cost_style.wrap_width / 2) as f32 => {
            //         long_size
            //     }
            //     _ => first_cost_style.font_size,
            // };

            let first_y: i32 = ((layout.height / 2) as f32
                + first_cost_width
                + first_cost_style.margin_top as f32)
                .round() as i32;

            draw_text_rotated_270(
                canvas,
                &first_cost,
                first_cost_style.x,
                first_y,
                first_cost_font_data,
                layout.font_sizes.split_cost,
                first_cost_style.letter_spacing,
                first_cost_style.wrap_width,
            );
        }

        if let Some(second_cost) = second_cost_opt {
            let second_cost_style = &layout.split_second_cost;
            let second_cost_font_data = layout.font_data(second_cost_style.font);

            let second_cost_width = layout.text_width(&second_cost, second_cost_style);

            // let second_cost_font_size = match second_cost_style.small_text_font_size {
            //     Some(long_size)
            //         if second_cost_width > (second_cost_style.wrap_width / 2) as f32 =>
            //     {
            //         long_size
            //     }
            //     _ => second_cost_style.font_size,
            // };

            let second_y: i32 =
                (second_cost_width + second_cost_style.margin_top as f32).round() as i32;

            draw_text_rotated_270(
                canvas,
                &second_cost,
                second_cost_style.x,
                second_y,
                second_cost_font_data,
                layout.font_sizes.split_cost,
                second_cost_style.letter_spacing,
                second_cost_style.wrap_width,
            );
        }

        Ok(())
    }
}

/// Renders split card art
pub struct SplitCardArtRenderer;
#[async_trait]
impl ElementRenderer for SplitCardArtRenderer {
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
            match card.is_room() {
                /* TODO just render a single piece of artwork for Room type cards

                Example: Walk-In Closet // Forgotten Cellar
                https://cards.scryfall.io/art/front/0/a/0adcd4e5-d542-4293-8774-ace2305ef820.webp?1783909446
                https://api.scryfall.com/cards/0adcd4e5-d542-4293-8774-ace2305ef820
                */
                true => {
                    let room_card_art = CardArtPipeline::process(
                        card_art_img.rotate270(),
                        layout.split_room_art.max_width,
                        layout.split_room_art.max_height,
                    );

                    let room_art_scale = (layout.split_room_art.max_width as f64
                        / room_card_art.width() as f64)
                        .min(
                            layout.split_room_art.max_height as f64 / room_card_art.height() as f64,
                        )
                        .min(1.0);

                    let room_art_render_width =
                        (room_card_art.width() as f64 * room_art_scale) as u32;

                    let room_art_render_height =
                        (room_card_art.height() as f64 * room_art_scale) as u32;

                    let resized_room_art = imageops::resize(
                        &room_card_art,
                        room_art_render_width,
                        room_art_render_height,
                        imageops::FilterType::Lanczos3,
                    );

                    let centered_y = (canvas
                        .height()
                        .saturating_sub(layout.split_room_art.max_height))
                        / 2;

                    // Optionally center the image inside its bounding box as well
                    let inner_offset_y = (layout
                        .split_room_art
                        .max_height
                        .saturating_sub(room_art_render_height))
                        / 2;

                    imageops::overlay(
                        canvas,
                        &resized_room_art,
                        layout.split_room_art.x,
                        (centered_y + inner_offset_y) as i64,
                    );
                }
                false => {
                    let half_width = card_art_img.width() / 2;
                    let card_art_first_img = card_art_img
                        .crop_imm(0, 0, half_width, card_art_img.height())
                        .rotate270();

                    let card_art_second_img = card_art_img
                        .crop_imm(half_width, 0, card_art_img.width(), card_art_img.height())
                        .rotate270();

                    card_art_first_img.save("/tmp/split_1_card_back_art.png")?;

                    card_art_second_img.save("/tmp/split_2_card_back_art.png")?;

                    let first_art = CardArtPipeline::process(
                        card_art_first_img,
                        layout.split_first_art.max_width,
                        layout.split_first_art.max_height,
                    );
                    let first_img_scale = (layout.split_first_art.max_width as f64
                        / first_art.width() as f64)
                        .min(layout.split_first_art.max_height as f64 / first_art.height() as f64)
                        .min(1.0);
                    let first_img_render_width =
                        (first_art.width() as f64 * first_img_scale) as u32;
                    let first_img_render_height =
                        (first_art.height() as f64 * first_img_scale) as u32;

                    let resized_first_art = imageops::resize(
                        &first_art,
                        first_img_render_width,
                        first_img_render_height,
                        imageops::FilterType::Lanczos3,
                    );

                    let first_img_bottom_half_height = layout.height / 2;
                    let first_imgbottom_half_center_y =
                        (layout.height / 2) + (first_img_bottom_half_height / 2);

                    let first_img_draw_y =
                        (first_imgbottom_half_center_y - (first_img_render_height / 2)) as i64;

                    imageops::overlay(
                        canvas,
                        &resized_first_art,
                        layout.split_first_art.x,
                        first_img_draw_y,
                    );

                    let second_art = CardArtPipeline::process(
                        card_art_second_img,
                        layout.split_second_art.max_width,
                        layout.split_second_art.max_height,
                    );

                    let second_img_scale = (layout.split_second_art.max_width as f64
                        / second_art.width() as f64)
                        .min(layout.split_second_art.max_height as f64 / second_art.height() as f64)
                        .min(1.0);

                    let second_img_render_width =
                        (second_art.width() as f64 * second_img_scale) as u32;
                    let second_img_render_height =
                        (second_art.height() as f64 * second_img_scale) as u32;

                    let resized_second_art = imageops::resize(
                        &second_art,
                        second_img_render_width,
                        second_img_render_height,
                        imageops::FilterType::Lanczos3,
                    );

                    let top_half_center_y = layout.height / 4;

                    let second_img_draw_y =
                        (top_half_center_y - (second_img_render_height / 2)) as i64;

                    imageops::overlay(
                        canvas,
                        &resized_second_art,
                        layout.split_second_art.x,
                        second_img_draw_y,
                    );
                }
            }
        }

        Ok(())
    }
}

/// Renders type line
pub struct SplitTypeLineRenderer;
#[async_trait]
impl ElementRenderer for SplitTypeLineRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let first_type_line_opt = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(0))
            .map(|face| face.type_line.clone())
            .unwrap_or_else(|| card.core.type_line.clone());

        let second_type_line_opt = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(1))
            .map(|face| face.type_line.clone())
            .unwrap_or_else(|| card.core.type_line.clone());

        if let Some(first_type_line) = first_type_line_opt {
            let first_type_style = &layout.split_first_type_line;
            let first_type_font_data = layout.font_data(first_type_style.font);
            let first_type_height = layout.wrapped_text_height(&first_type_line, first_type_style);
            let first_type_adjusted_y = (layout.height - first_type_style.margin_bottom) as i32;

            let first_type_end_x = draw_text_rotated_270(
                canvas,
                &first_type_line,
                first_type_style.x,
                // type_style.y,
                first_type_adjusted_y,
                // first_type_style.y,
                first_type_font_data,
                first_type_style
                    .small_text_font_size
                    .unwrap_or(first_type_style.font_size),
                first_type_style.letter_spacing,
                first_type_style.wrap_width,
            );
            layout.split_first_line_mid_point = first_type_style.x - (first_type_height / 2);
            layout.split_first_line_end_x = first_type_end_x;
        }

        if let Some(second_type_line) = second_type_line_opt {
            let second_type_style = &layout.split_second_type_line;
            let second_type_font_data = layout.font_data(second_type_style.font);
            let second_type_height =
                layout.wrapped_text_height(&second_type_line, second_type_style);
            let second_type_adjusted_y =
                ((layout.height / 2) - second_type_style.margin_bottom) as i32;

            let second_type_end_x = draw_text_rotated_270(
                canvas,
                &second_type_line,
                second_type_style.x,
                // type_style.y,
                second_type_adjusted_y,
                // first_type_style.y,
                second_type_font_data,
                second_type_style
                    .small_text_font_size
                    .unwrap_or(second_type_style.font_size),
                second_type_style.letter_spacing,
                second_type_style.wrap_width,
            );
            layout.split_second_line_mid_point = second_type_style.x - (second_type_height / 2);
            layout.split_second_line_end_x = second_type_end_x;
        }

        // TODO Write the renderer for Split card type lines

        Ok(())
    }
}

/// Renders set icon
pub struct SplitSetIconRenderer;
#[async_trait]
impl ElementRenderer for SplitSetIconRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        _face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if card.core.set_icon_svg_uri.is_some() {
            debug!("Loading set icon from Scryfall");
            let client = ScryfallClient::new(Some(SCRYFALL_USER_AGENT))?;
            let set = ScryfallSet::from_id(&card.core.set_id, &client).await?;
            let svg_data = set.get_svg_bytes(&client).await?;

            let first_set_icon = &layout.split_first_set_icon;

            draw_svg_rotated_270(
                canvas,
                &svg_data,
                layout.split_first_line_mid_point as u32,
                (layout.height / 2) + first_set_icon.margin_bottom as u32,
                first_set_icon.max_width,
                first_set_icon.max_height,
            )?;

            let second_set_icon = &layout.split_second_set_icon;

            draw_svg_rotated_270(
                canvas,
                &svg_data,
                layout.split_second_line_mid_point as u32,
                second_set_icon.margin_bottom as u32,
                second_set_icon.max_width,
                second_set_icon.max_height,
            )?;
        }

        Ok(())
    }
}

/// Renders oracle text
pub struct SplitOracleTextRenderer;
#[async_trait]
impl ElementRenderer for SplitOracleTextRenderer {
    async fn render(
        &self,
        card: &OracleScryfallCard,
        face: Option<&CardFace>,
        canvas: &mut RgbImage,
        layout: &mut Layout,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // TODO Finish basic renderer for Split card oracle text

        let mut first_oracle_opt = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(0))
            .map(|face| face.oracle_text.clone())
            .unwrap_or_else(|| card.core.oracle_text.clone());

        let mut second_oracle_opt = card
            .core
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(1))
            .map(|face| face.oracle_text.clone())
            .unwrap_or_else(|| card.core.oracle_text.clone());

        if card.has_keyword("Fuse") {
            let fuse_text = "\nFuse (You may cast one or both halves of this card from your hand.)";

            if let Some(ref mut text) = first_oracle_opt {
                *text = text.replace(fuse_text, "");
            }
            if let Some(ref mut text) = second_oracle_opt {
                *text = text.replace(fuse_text, "");
            }

            let fuse_reminder_style = &layout.split_fuse_reminder_text;
            let fuse_reminder_font_data = layout.font_data(fuse_reminder_style.font);

            // Total rendered length along the card's Y axis
            let total_text_width = layout.wrapped_text_width(fuse_text, fuse_reminder_style);

            let center_y = layout.height as i32 / 2;

            let centered_baseline_y = (center_y as f32 + (total_text_width / 2.0)) as i32;

            draw_text_rotated_270(
                canvas,
                fuse_text,
                (layout.width - fuse_reminder_style.margin_right) as i32,
                centered_baseline_y,
                fuse_reminder_font_data,
                fuse_reminder_style.font_size,
                fuse_reminder_style.letter_spacing,
                fuse_reminder_style.wrap_width,
            );
        }

        if let Some(first_oracle_text) = first_oracle_opt {
            let oracle_style = &layout.split_first_oracle_text;
            let oracle_font_data = layout.font_data(oracle_style.font);
            let rules_x = oracle_style.x.max(layout.split_first_line_end_x);
            let oracle_width = layout.wrapped_text_width(&first_oracle_text, oracle_style);
            let font_size = match oracle_style.large_text_font_size {
                Some(long_size) if oracle_width > oracle_style.wrap_width as f32 => long_size,
                _ => oracle_style.font_size,
            };

            let center_y = (canvas.height() as f32 * 3.0) / 4.0;
            let baseline_y = (center_y + (oracle_width / 2.0).round()) as i32;
            draw_text_rotated_270(
                canvas,
                &first_oracle_text,
                rules_x,
                // oracle_style.y,
                baseline_y,
                oracle_font_data,
                font_size,
                oracle_style.letter_spacing,
                oracle_style.wrap_width,
            );
        }

        if let Some(second_oracle_string) = second_oracle_opt {
            let oracle_style = &layout.split_second_oracle_text;
            let oracle_font_data = layout.font_data(oracle_style.font);
            let rules_x = oracle_style.x.max(layout.split_first_line_end_x);
            let oracle_width = layout.wrapped_text_width(&second_oracle_string, oracle_style);
            let font_size = match oracle_style.large_text_font_size {
                Some(long_size) if oracle_width > oracle_style.wrap_width as f32 => long_size,
                _ => oracle_style.font_size,
            };

            let center_y = (canvas.height() as f32) / 4.0;
            let baseline_y = (center_y + (oracle_width / 2.0).round()) as i32;
            draw_text_rotated_270(
                canvas,
                &second_oracle_string,
                rules_x,
                // oracle_style.y,
                baseline_y,
                oracle_font_data,
                font_size,
                oracle_style.letter_spacing,
                oracle_style.wrap_width,
            );
        }

        /*
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
        */
        Ok(())
    }
}
