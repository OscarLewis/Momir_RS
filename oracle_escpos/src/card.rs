use crate::{
    art::CardArtPipeline,
    layout::{self, BorderStyle, Layout},
    render::{
        BorderWrapConfig, draw_border, draw_svg, draw_text, draw_text_around_border, text_width,
    },
};
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
        Self {
            scryfall_id: card.core.id.clone(),
            card,
        }
    }

    /// Generates the card image and saves it as a PNG
    pub async fn generate(self, out_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        // Set Fonts
        let serif_font_path =
            "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/fonts/Mplantin.ttf";

        let sanserif_font_path =
            "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/fonts/tahoma.ttf";

        let (serif_font, sanserif_font) = Layout::load_fonts(serif_font_path, sanserif_font_path)?;

        // Extract Option<Strings>
        let oracle_text = self.card.core.oracle_text.unwrap_or_default();
        let type_line = self.card.core.type_line.unwrap_or_default();

        let mut layout = Layout {
            serif_font,
            sanserif_font,
            ..Layout::default()
        };

        // Create base image after layout is created.
        let mut card_img = RgbImage::from_pixel(layout.width, layout.height, Rgb([255, 255, 255]));

        info!(
            scryfall_id = %self.scryfall_id,
            card_name = %self.card.core.name,
            width = layout.width,
            height = layout.height,
            layout= %self.card.core.layout,
            "Generating card image"
        );

        //
        // Card art
        //
        let card_art = if let Some(image_uris) = &self.card.print.image_uris {
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

            // let art = card_art_img
            //     .resize(
            //         layout.art.max_width,
            //         layout.art.max_height,
            //         imageops::FilterType::Lanczos3,
            //     )
            //     .grayscale()
            //     .to_rgb8();

            // Calculate the scaled width after fitting into max dimensions
            let scale = (layout.art.max_width as f64 / ca_width as f64)
                .min(layout.art.max_height as f64 / ca_height as f64)
                .min(1.0); // don't upscale if smaller

            let render_width = (ca_width as f64 * scale) as i64;

            // Center using the actual rendered width
            let draw_x = (layout.art.x as i64) + ((layout.art.max_width as i64) - render_width) / 2;

            imageops::overlay(&mut card_img, &art, draw_x, layout.art.y);

            // imageops::overlay(&mut card_img, &art, layout.art.x, layout.art.y);
        }
        // End of Cart Art render block

        //
        // Card name
        //

        // Is this card from an unset?
        let is_funny = self.card.core.set_type == "funny";

        // Determine border wrapping style (Standard, SemiWrap, or FullWrap)
        // This is really important and has far reaching impacts beyond just name
        // BorderStyles modify the default style to shift elements around
        let border_style = {
            let name_style = &layout.name;
            let name_font_data = layout.font_data(name_style.font);
            let name_width = layout.text_width(&self.card.core.name, name_style);

            if let Some(long_text_font_size) = name_style.long_text_font_size {
                if name_width > name_style.wrap_width as f32 {
                    let total_text_len = text_width(
                        &self.card.core.name,
                        name_font_data,
                        long_text_font_size,
                        0.5,
                    );
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

        // Update layout state
        layout.border_style = border_style;

        // Apply layout modifications from the local variable
        border_style.apply_layout_adjustments(&mut layout);

        // Proceed with rendering standard or border-wrap text
        let name_style = &layout.name;
        let name_font_data = layout.font_data(name_style.font);
        let name_width = layout.text_width(&self.card.core.name, name_style);

        match layout.border_style {
            BorderStyle::FullWrap | BorderStyle::SemiWrap => {
                draw_text_around_border(
                    &mut card_img,
                    &self.card.core.name,
                    name_font_data,
                    name_style.font_size,
                    name_style.letter_spacing,
                    &layout.border_path,
                );
            }
            BorderStyle::Standard | BorderStyle::LongName => {
                draw_text(
                    &mut card_img,
                    &self.card.core.name,
                    name_style.x,
                    name_style.y,
                    name_font_data,
                    name_style.font_size,
                    name_style.letter_spacing,
                    name_style.wrap_width,
                );
            }
        }
        // End of Card Name render block

        //
        // Mana Cost line
        //
        let mana_cost_style = &layout.cost;
        let mana_cost_font_data = layout.font_data(mana_cost_style.font);
        if let Some(mana_cost) = &self.card.core.mana_cost {
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
                &mut card_img,
                &mana_cost,
                cost_x,
                mana_cost_style.y,
                mana_cost_font_data,
                cost_font_size,
                mana_cost_style.letter_spacing,
                mana_cost_style.wrap_width,
            );
        }

        //
        // Type line
        //
        let type_style = &layout.type_line;
        let type_font_data = layout.font_data(type_style.font);

        debug!(font_size = type_style.font_size, "Rendering type line");

        let type_line_end_y = draw_text(
            &mut card_img,
            &type_line,
            type_style.x,
            type_style.y,
            type_font_data,
            type_style.font_size,
            type_style.letter_spacing,
            type_style.wrap_width,
        );

        //
        // Oracle text
        //
        let rules_style = &layout.rules;
        let rules_font_data = layout.font_data(rules_style.font);
        // Calculate oracle start position: use fixed rules_style.y if type line didn't wrap,
        // or chain directly off type_line_end_y if it wrapped lower.
        let rules_y = rules_style.y.max(type_line_end_y);

        debug!(
            font_size = rules_style.font_size,
            oracle_text_length = oracle_text.len(),
            "Rendering oracle text"
        );

        draw_text(
            &mut card_img,
            &oracle_text,
            rules_style.x,
            rules_y,
            rules_font_data,
            rules_style.font_size,
            rules_style.letter_spacing,
            rules_style.wrap_width,
        );

        //
        // Artist Credit
        //
        if let Some(artist_name) = &self.card.print.artist {
            let artist_style = &layout.artist;
            let artist_font_data = layout.font_data(artist_style.font);

            draw_text(
                &mut card_img,
                &format!("Art by {}", &artist_name),
                artist_style.x,
                artist_style.y,
                artist_font_data,
                artist_style.font_size,
                artist_style.letter_spacing,
                artist_style.wrap_width,
            );
        }

        //
        // Set icon
        //
        if self.card.core.set_icon_svg_uri.is_some() {
            debug!("Loading set icon from Scryfall");

            let client = ScryfallClient::new()?;

            let set = ScryfallSet::from_id(&self.card.core.set_id, &client).await?;

            let svg_data = set.get_svg_bytes(&client).await?;

            let set_icon = &layout.set_icon;

            draw_svg(
                &mut card_img,
                &svg_data,
                set_icon.x,
                set_icon.y,
                set_icon.width,
                set_icon.height,
            )?;
        }

        //
        // Set code
        //
        let metadata_style = &layout.set_code;
        let metadata_font_data = layout.font_data(metadata_style.font);

        draw_text(
            &mut card_img,
            &format!(
                "{} {}",
                self.card.core.set.to_uppercase(),
                self.card.print.collector_number
            ),
            metadata_style.x,
            metadata_style.y,
            metadata_font_data,
            metadata_style.font_size,
            metadata_style.letter_spacing,
            metadata_style.wrap_width,
        );

        //
        // Power / toughness
        //
        let pow_tough_style = &layout.pow_tough_style;
        let pow_tough_font_data = layout.font_data(pow_tough_style.font);

        if let (Some(power), Some(toughness)) = (&self.card.core.power, &self.card.core.toughness) {
            debug!(
                power = %power,
                toughness = %toughness,
                "Rendering power and toughness"
            );

            draw_text(
                &mut card_img,
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

        //
        // Border
        //
        debug!("Rendering card border");

        draw_border(&mut card_img);

        //
        // Save
        //
        card_img.save(out_path)?;

        info!(
            scryfall_id = %self.scryfall_id,
            "Card image generated successfully"
        );

        Ok(())
    }
}
