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

        // --- First Name Setup ---
        let first_name_style = &layout.split_first_name;
        let first_font_data = layout.font_data(first_name_style.font);
        let first_name_width = layout.text_width(&first_name, first_name_style);

        let first_font_size = match first_name_style.small_text_font_size {
            Some(long_size) if first_name_width > first_name_style.wrap_width as f32 => long_size,
            _ => first_name_style.font_size,
        };

        let first_y: i32 = (layout.height - first_name_style.margin_bottom) as i32;

        // --- Second Name Setup ---
        let second_name_style = &layout.split_second_name;
        let second_font_data = layout.font_data(second_name_style.font);
        // Fixed: calculate width using second_name instead of first_name
        let second_name_width = layout.text_width(&second_name, second_name_style);

        let second_font_size = match second_name_style.small_text_font_size {
            Some(long_size) if second_name_width > second_name_style.wrap_width as f32 => long_size,
            _ => second_name_style.font_size,
        };

        let second_y: i32 =
            (second_name_style.margin_top as f32 + second_name_width).round() as i32;

        // --- Render Calls ---
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

        draw_horizontal_line(
            canvas,
            20,
            (layout.height / 2) as i32,
            (layout.width - 40) as i32,
            2,
        );
        Ok(())
    }
}
