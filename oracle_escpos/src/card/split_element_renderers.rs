use crate::{
    card::element_renderers::ElementRenderer,
    layout::Layout,
    render::{draw_horizontal_line, draw_text_rotated_270},
};
use async_trait::async_trait;
use image::RgbImage;
use scryfall_oracle::{CardFace, OracleScryfallCard};

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
        // Fixed: calculate width using second_name instead of first_name
        let second_name_width = layout.text_width(&second_name, second_name_style);

        let second_font_size = match second_name_style.small_text_font_size {
            Some(long_size) if second_name_width > second_name_style.wrap_width as f32 => long_size,
            _ => second_name_style.font_size,
        };

        let second_y: i32 = ((layout.height / 2) - second_name_style.margin_bottom) as i32;

        // Render Calls
        draw_horizontal_line(
            canvas,
            20,
            (layout.height / 2) as i32,
            (layout.width - 40) as i32,
            2,
        );

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

/*
/// Renders split card art
pub struct SplitBackCardArtRenderer;
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
*/
