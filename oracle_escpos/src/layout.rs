use std::path::PathBuf;

use tracing::debug;

use crate::render::text_width;

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub x: i32,
    pub y: i32,
    pub font: Font,
    pub margin_left: u32,
    pub margin_right: u32,
    pub font_size: f32,
    pub long_text_font_size: Option<f32>,
    pub letter_spacing: f32,
    pub wrap_width: i32,
}
#[derive(Debug, Clone, Copy)]
pub enum Font {
    Serif,
    Sanserif,
}
impl Default for TextStyle {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            font_size: 0.0,
            margin_left: 0,
            margin_right: 0,
            font: Font::Sanserif,
            long_text_font_size: None,
            letter_spacing: 0.0,
            wrap_width: 372,
        }
    }
}
#[derive(Debug, Clone)]
pub struct FontSizes {
    pub name: f32,
    pub long_name: f32,
    pub type_line: f32,
    pub rules: f32,
    pub flavor: f32,
    pub pow_tough: f32,
    pub set_code: f32,
    pub cost: f32,
    pub long_cost: f32,
    pub artist: f32,
}

impl Default for FontSizes {
    fn default() -> Self {
        Self {
            name: 34.0,
            long_name: 24.0,
            type_line: 18.0,
            rules: 16.0,
            pow_tough: 28.0,
            flavor: 17.0,
            set_code: 14.0,
            artist: 14.0,
            cost: 16.0,
            long_cost: 12.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub width: u32,
    pub height: u32,
    pub font_sizes: FontSizes,
    pub sanserif_font: Vec<u8>,
    pub serif_font: Vec<u8>,
    pub art: ArtLayout,
    pub name: TextStyle,
    pub cost: TextStyle,
    pub set_icon: SvgLayout,
    pub type_line: TextStyle,
    pub rules: TextStyle,
    pub flavor: TextStyle,
    pub set_code: TextStyle,
    pub artist: TextStyle,
    pub pow_tough_style: TextStyle,
}

#[derive(Debug, Clone)]
pub struct ArtLayout {
    pub x: i64,
    pub y: i64,
    pub max_width: u32,
    pub max_height: u32,
}
impl Default for Layout {
    fn default() -> Self {
        let fonts = FontSizes::default();

        Self {
            width: 412,
            height: 576,
            set_icon: SvgLayout {
                x: 20,
                y: 496,
                width: 50,
                height: 50,
            },
            serif_font: Vec::new(),
            sanserif_font: Vec::new(),

            font_sizes: fonts.clone(),

            art: ArtLayout {
                x: 60,
                y: 76,
                max_width: 300,
                max_height: 200,
            },

            name: TextStyle {
                x: 20,
                y: 35,
                font: Font::Serif,
                font_size: fonts.name,
                long_text_font_size: Some(fonts.long_name),
                letter_spacing: 0.0,
                wrap_width: 372,
                ..Default::default()
            },

            cost: TextStyle {
                // x: 260,
                y: 55,
                font: Font::Serif,
                font_size: fonts.cost,
                letter_spacing: 1.0,
                wrap_width: 120,
                margin_right: 26,
                ..Default::default()
            },

            type_line: TextStyle {
                x: 20,
                y: 310,
                font: Font::Sanserif,
                font_size: fonts.type_line,
                wrap_width: 372,
                ..Default::default()
            },

            rules: TextStyle {
                x: 20,
                y: 344,
                font: Font::Sanserif,
                font_size: fonts.rules,
                letter_spacing: 1.0,
                wrap_width: 372,
                ..Default::default()
            },

            flavor: TextStyle {
                x: 20,
                y: 0,
                font: Font::Serif,
                font_size: fonts.flavor,
                wrap_width: 372,
                ..Default::default()
            },

            set_code: TextStyle {
                x: 30,
                y: 566,
                font: Font::Sanserif,
                font_size: fonts.set_code,
                wrap_width: 372,
                ..Default::default()
            },

            artist: TextStyle {
                x: 140,
                y: 566,
                font: Font::Sanserif,
                font_size: fonts.artist,
                wrap_width: 372,
                letter_spacing: 1.0,
                ..Default::default()
            },

            pow_tough_style: TextStyle {
                x: 330,
                y: 566,
                font: Font::Serif,
                font_size: fonts.pow_tough,
                letter_spacing: 1.0,
                wrap_width: 62,
                ..Default::default()
            },
        }
    }
}

impl Layout {
    pub fn font_data(&self, font: Font) -> &[u8] {
        match font {
            Font::Serif => &self.serif_font,
            Font::Sanserif => &self.sanserif_font,
        }
    }
    pub fn load_fonts(
        serif_font_path: impl Into<PathBuf>,
        sanserif_font_path: impl Into<PathBuf>,
    ) -> Result<(Vec<u8>, Vec<u8>), std::io::Error> {
        let serif_font_path = serif_font_path.into();
        let sanserif_font_path = sanserif_font_path.into();

        debug!(path = ?serif_font_path, "Loading serif font");
        let serif_font = std::fs::read(&serif_font_path)?;

        debug!(path = ?sanserif_font_path, "Loading sans-serif font");
        let sanserif_font = std::fs::read(&sanserif_font_path)?;

        Ok((serif_font, sanserif_font))
    }
    pub fn text_width(&self, text: &str, style: &TextStyle) -> f32 {
        text_width(
            text,
            self.font_data(style.font),
            style.font_size,
            style.letter_spacing,
        )
    }
}
#[derive(Debug, Clone)]
pub struct SvgLayout {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
