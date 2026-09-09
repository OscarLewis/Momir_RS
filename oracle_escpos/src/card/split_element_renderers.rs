use crate::{
    art::CardArtPipeline,
    card::element_renderers::ElementRenderer,
    layout::Layout,
    render::{draw_horizontal_line, draw_text_rotated_270},
};
use async_trait::async_trait;
use image::{RgbImage, imageops};
use scryfall_oracle::{CardFace, OracleScryfallCard, ScryfallClient};
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
        let first_name_width = layout.text_width(&first_name, first_name_style);

        let first_font_size = match first_name_style.small_text_font_size {
            Some(long_size) if first_name_width > first_name_style.wrap_width as f32 => long_size,
            _ => first_name_style.font_size,
        };

        let first_y: i32 = (layout.height - first_name_style.margin_bottom) as i32;

        // Second Name Setup
        let second_name_style = &layout.split_second_name;
        let second_font_data = layout.font_data(second_name_style.font);
        let second_name_width = layout.text_width(&second_name, second_name_style);

        let second_font_size = match second_name_style.small_text_font_size {
            Some(long_size) if second_name_width > second_name_style.wrap_width as f32 => long_size,
            _ => second_name_style.font_size,
        };

        let second_y: i32 = ((layout.height / 2) - second_name_style.margin_bottom) as i32;

        // Render Calls
        if !card.has_type_word("Room") {
            draw_horizontal_line(
                canvas,
                20,
                (layout.height / 2) as i32,
                (layout.width - 40) as i32,
                2,
            );
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

            let first_name_width = layout.text_width(&first_cost, first_cost_style);
            // First Name Setup

            let first_cost_font_size = match first_cost_style.small_text_font_size {
                Some(long_size) if first_name_width > first_cost_style.wrap_width as f32 => {
                    long_size
                }
                _ => first_cost_style.font_size,
            };

            let first_y: i32 = ((layout.height / 2) as f32
                + first_name_width
                + first_cost_style.margin_top as f32)
                .round() as i32;

            draw_text_rotated_270(
                canvas,
                &first_cost,
                first_cost_style.x,
                first_y,
                first_cost_font_data,
                first_cost_font_size,
                first_cost_style.letter_spacing,
                first_cost_style.wrap_width,
            );
        }

        if let Some(second_cost) = second_cost_opt {
            let second_cost_style = &layout.split_second_cost;
            let second_cost_font_data = layout.font_data(second_cost_style.font);

            let second_name_width = layout.text_width(&second_cost, second_cost_style);

            let second_cost_font_size = match second_cost_style.small_text_font_size {
                Some(long_size) if second_name_width > second_cost_style.wrap_width as f32 => {
                    long_size
                }
                _ => second_cost_style.font_size,
            };

            let second_y: i32 =
                (second_name_width + second_cost_style.margin_top as f32).round() as i32;

            draw_text_rotated_270(
                canvas,
                &second_cost,
                second_cost_style.x,
                second_y,
                second_cost_font_data,
                second_cost_font_size,
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
            let first_img_render_width = (first_art.width() as f64 * first_img_scale) as u32;
            let first_img_render_height = (first_art.height() as f64 * first_img_scale) as u32;

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

            let second_img_render_width = (second_art.width() as f64 * second_img_scale) as u32;
            let second_img_render_height = (second_art.height() as f64 * second_img_scale) as u32;

            let resized_second_art = imageops::resize(
                &second_art,
                second_img_render_width,
                second_img_render_height,
                imageops::FilterType::Lanczos3,
            );

            let top_half_center_y = layout.height / 4;

            let second_img_draw_y = (top_half_center_y - (second_img_render_height / 2)) as i64;

            imageops::overlay(
                canvas,
                &resized_second_art,
                layout.split_second_art.x,
                second_img_draw_y,
            );
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
            let first_type_width = layout.text_width(&first_type_line, first_type_style);
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
        }

        if let Some(second_type_line) = second_type_line_opt {
            let second_type_style = &layout.split_second_type_line;
            let second_type_font_data = layout.font_data(second_type_style.font);
            let second_type_width = layout.text_width(&second_type_line, second_type_style);
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
        }

        // TODO Write the renderer for Split card type lines

        /*
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
        */

        Ok(())
    }
}
