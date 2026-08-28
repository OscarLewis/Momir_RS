use crate::render::text_width;
use std::path::PathBuf;
use tracing::debug;

#[derive(Debug, Clone, Copy)]
pub enum Font {
    Serif,
    Sanserif,
}

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
    pub small_rules: f32,
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
            long_name: 22.0,
            type_line: 18.0,
            small_rules: 15.0,
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
pub struct ArtLayout {
    pub x: i64,
    pub y: i64,
    pub max_width: u32,
    pub max_height: u32,
}

#[derive(Debug, Clone)]
pub struct SvgLayout {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Layout dimensions and coordinate bounds for wrapping text along card borders.
#[derive(Clone, Copy, Debug)]
pub struct BorderPathLayout {
    /// X coordinate baseline for vertical text along the left border.
    pub left_x: i32,
    /// Y coordinate baseline for horizontal text along the top border.
    pub top_y: i32,
    /// X coordinate baseline for vertical text along the right border.
    pub right_x: i32,
    /// Starting X coordinate where text rendering begins on the top edge.
    pub top_start_x: i32,
    /// Ending X coordinate where text rendering stops on the top edge before wrapping.
    pub top_end_x: i32,
    /// Y coordinate at the bottom boundary of the left side path.
    pub left_side_bottom_y: i32,
    /// Y coordinate at the top boundary of the left side path.
    pub left_side_top_y: i32,
    /// Y coordinate at the top boundary of the right side path.
    pub right_side_top_y: i32,
    /// Y coordinate at the bottom boundary of the right side path.
    pub right_side_bottom_y: i32,
    /// Starting X coordinate on the right side where text begins wrapping leftward along the bottom border.
    pub bottom_x_start: i32,
    /// Ending X coordinate where text stops on the bottom border before returning up the left edge.
    pub bottom_x_end: i32,
    /// Y coordinate baseline for 180-degree rotated text along the bottom border.
    pub bottom_y: i32,
    /// Y coordinate baseline for starting the return loop back up the left border.
    pub left_side_return_bottom_y: i32,
    /// Y coordinate baseline for ending the return loop up the left border.
    pub left_side_return_top_y: i32,
}

impl Default for BorderPathLayout {
    /// Provides baseline coordinate presets for standard card layout dimensions.
    fn default() -> Self {
        Self {
            left_x: 20,
            top_y: 28,
            right_x: 390,
            top_start_x: 25,
            top_end_x: 380,
            left_side_bottom_y: 330,
            left_side_top_y: 25,
            right_side_top_y: 25,
            right_side_bottom_y: 535,
            bottom_x_start: 375,
            bottom_x_end: 25,
            bottom_y: 546,
            left_side_return_bottom_y: 535,
            left_side_return_top_y: 360,
        }
    }
}
impl BorderPathLayout {
    /// Distance along perimeter where text reaches the bottom edge.
    pub fn bottom_threshold(&self) -> f32 {
        let left_height = (self.left_side_bottom_y - self.left_side_top_y) as f32;
        let top_width = (self.top_end_x - self.top_start_x) as f32;
        let right_height = (self.right_side_bottom_y - self.right_side_top_y) as f32;

        left_height + top_width + right_height
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    Standard,
    FullWrap,
    SemiWrap,
    LongName,
}

impl BorderStyle {
    /// Applies layout margin and dimension shifts based on the border style.
    /// These automatically override the default Standard style.
    pub fn apply_layout_adjustments(&self, layout: &mut Layout) {
        let fonts = FontSizes::default();
        match self {
            BorderStyle::FullWrap => {
                layout.type_line.x = 35;
                layout.rules.x = 35;
                layout.rules.wrap_width = 350;
                layout.rules.font_size = fonts.small_rules;
                layout.rules.letter_spacing = 1.5;
                layout.set_icon.x = 43;
                layout.set_icon.y = 468;
                layout.set_code.x = 40;
                layout.set_code.y = 530;
                layout.set_code.letter_spacing = 1.0;
                layout.artist.x = 130;
                layout.artist.y = 530;
                layout.artist.letter_spacing = 1.5;
                layout.pow_tough_style.x = 330;
                layout.pow_tough_style.y = 530;
                layout.name.font_size = fonts.long_name;
                layout.name.letter_spacing = 0.5;
                layout.cost.margin_right = 35;
            }
            BorderStyle::Standard => {
                // Default margins stay intact
            }
            BorderStyle::SemiWrap => {
                layout.type_line.x = 35;
                layout.name.font_size = fonts.long_name;
                layout.name.letter_spacing = 1.0;
                layout.rules.x = 35;
                layout.cost.margin_right = 35;
                layout.rules.wrap_width = 350;
            }
            BorderStyle::LongName => {
                layout.name.font_size = fonts.long_name;
                layout.name.letter_spacing = 1.0;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub width: u32,
    pub height: u32,
    pub border_style: BorderStyle,
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
    pub border_path: BorderPathLayout,
}

impl Default for Layout {
    fn default() -> Self {
        let fonts = FontSizes::default();

        Self {
            width: 416,
            height: 576,
            border_style: BorderStyle::Standard,
            border_path: BorderPathLayout::default(),

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
                letter_spacing: 1.0,
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

            set_icon: SvgLayout {
                x: 36,
                y: 496,
                width: 50,
                height: 50,
            },

            set_code: TextStyle {
                x: 30,
                y: 566,
                font: Font::Sanserif,
                font_size: fonts.set_code,
                wrap_width: 372,
                letter_spacing: 1.5,
                ..Default::default()
            },

            artist: TextStyle {
                x: 140,
                y: 566,
                font: Font::Sanserif,
                font_size: fonts.artist,
                wrap_width: 372,
                letter_spacing: 1.5,
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
