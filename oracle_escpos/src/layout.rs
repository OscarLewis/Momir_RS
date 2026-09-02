use crate::{
    card::fonts::{Font, fonts},
    render::{text_width, wrapped_text_height, wrapped_text_width},
};

#[derive(Debug, Clone)]
pub struct TextStyle {
    pub x: i32,
    pub y: i32,
    pub font: Font,
    pub margin_left: u32,
    pub margin_right: u32,
    pub max_length: Option<i32>,
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
            max_length: None,
            font_size: 0.0,
            margin_left: 0,
            margin_right: 0,
            font: Font::Sansserif,
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
    pub adventure_type_line: f32,
    pub adventure_name: f32,
    pub long_adventure_name: f32,
    pub rules: f32,
    pub small_rules: f32,
    pub flavor: f32,
    pub pow_tough: f32,
    pub set_code: f32,
    pub cost: f32,
    pub long_cost: f32,
    pub artist: f32,
    pub artist_small: f32,
    pub loyalty: f32,
    pub meld_type_line: f32,
    pub meld_oracle: f32,
}

impl Default for FontSizes {
    fn default() -> Self {
        Self {
            adventure_name: 16.0,
            adventure_type_line: 16.0,
            artist_small: 14.0,
            artist: 16.0,
            cost: 20.0,
            flavor: 17.0,
            long_adventure_name: 14.0,
            long_cost: 18.0,
            long_name: 23.0,
            loyalty: 34.0,
            meld_oracle: 20.0,
            meld_type_line: 28.0,
            name: 34.0,
            pow_tough: 30.0,
            rules: 16.0,
            set_code: 16.0,
            small_rules: 15.0,
            type_line: 18.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ArtLayout {
    pub x: i64,
    pub y: i64,
    pub margin_right: Option<i64>,
    pub max_width: u32,
    pub max_height: u32,
}

#[derive(Debug, Clone)]
pub struct SvgLayout {
    pub x: u32,
    pub y: u32,
    pub max_width: u32,
    pub max_height: u32,
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
pub enum WrapStyle {
    Standard,
    SemiWrap,
    FullWrap,
}

impl WrapStyle {
    /// Applies layout adjustments associated with the wrapping style.
    pub fn apply_layout_adjustments(self, layout: &mut Layout) {
        let fonts = FontSizes::default();

        match self {
            Self::Standard => {
                // Default layout stays intact.
            }

            Self::SemiWrap => {
                layout.type_line.x = 35;
                layout.rules.x = 35;
                layout.cost.margin_right = 35;
                layout.rules.wrap_width = 350;
            }

            Self::FullWrap => {
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
                layout.artist.font_size = fonts.artist_small;

                layout.pow_tough_style.x = 330;
                layout.pow_tough_style.y = 530;

                layout.cost.margin_right = 35;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameStyle {
    Standard,
    LongName,
}

impl NameStyle {
    /// Applies layout adjustments associated with the name style.
    pub fn apply_layout_adjustments(self, layout: &mut Layout) {
        let fonts = FontSizes::default();

        match self {
            Self::Standard => {
                // Default name layout stays intact.
            }

            Self::LongName => {
                layout.name.font_size = fonts.long_name;
                layout.name.letter_spacing = 1.0;
            }
        }
    }
}

/// Defines the complete layout and styling configuration for a card.
///
/// This struct contains all positioning, sizing, and font properties for various
/// card elements including name, mana cost, type line, oracle text, and more.
/// It supports different card styles through wrap and name style variants.
#[derive(Debug, Clone)]
pub struct Layout {
    pub adventure_mana_cost: TextStyle,
    pub adventure_name: TextStyle,
    pub adventure_oracle_text_alt_face: TextStyle,
    pub adventure_oracle_text_main_face: TextStyle,
    pub adventure_type_line: TextStyle,
    pub art: ArtLayout,
    pub artist: TextStyle,
    pub border_path: BorderPathLayout,
    pub cost: TextStyle,
    pub flavor: TextStyle,
    pub font_sizes: FontSizes,
    pub height: u32,
    pub meld_card_back_art: ArtLayout,
    pub meld_name: TextStyle,
    pub meld_type_line_end_x: i32,
    pub meld_type_line: TextStyle,
    pub meld_oracle: TextStyle,
    pub meld_set_code: TextStyle,
    pub meld_planeswalker_loyalty_shield: SvgLayout,
    pub name_style: NameStyle,
    pub name: TextStyle,
    pub planeswalker_loyalty_shield: SvgLayout,
    pub planeswalker_loyalty_style: TextStyle,
    pub pow_tough_style: TextStyle,
    pub rules: TextStyle,
    pub sanserif_font: &'static [u8],
    pub serif_font: &'static [u8],
    pub set_code: TextStyle,
    pub set_icon: SvgLayout,
    pub type_line_end_y: i32,
    pub type_line: TextStyle,
    pub width: u32,
    pub wrap_style: WrapStyle,
}

impl Default for Layout {
    fn default() -> Self {
        let fonts = FontSizes::default();

        Self {
            width: 416,
            height: 576,

            wrap_style: WrapStyle::Standard,
            name_style: NameStyle::Standard,

            border_path: BorderPathLayout::default(),

            font_sizes: fonts.clone(),

            art: ArtLayout {
                x: 60,
                y: 76,
                margin_right: None,
                max_width: 300,
                max_height: 200,
            },

            meld_card_back_art: ArtLayout {
                margin_right: Some(0),
                x: 0,
                y: 0,
                max_width: 396,
                max_height: 556,
            },

            meld_name: TextStyle {
                x: 50,
                y: 372,
                font: Font::Serif,
                font_size: fonts.name,
                long_text_font_size: Some(fonts.long_name),
                wrap_width: 372,
                ..Default::default()
            },

            name: TextStyle {
                x: 20,
                y: 35,
                font: Font::Serif,
                font_size: fonts.name,
                long_text_font_size: Some(fonts.long_name),
                wrap_width: 372,
                ..Default::default()
            },

            adventure_name: TextStyle {
                x: 20,
                y: 364,
                font: Font::Serif,
                font_size: fonts.adventure_name,
                long_text_font_size: Some(fonts.long_adventure_name),
                wrap_width: 372,
                ..Default::default()
            },

            cost: TextStyle {
                y: 55,
                font: Font::Sansserif,
                font_size: fonts.cost,
                wrap_width: 160,
                margin_right: 26,
                ..Default::default()
            },

            adventure_mana_cost: TextStyle {
                y: 384,
                font: Font::Sansserif,
                font_size: fonts.long_cost,
                wrap_width: 100,
                margin_left: 20,
                ..Default::default()
            },

            type_line: TextStyle {
                x: 20,
                y: 310,
                font: Font::Sansserif,
                font_size: fonts.type_line,
                wrap_width: 372,
                ..Default::default()
            },

            type_line_end_y: 20,

            meld_type_line: TextStyle {
                x: 40,
                y: 310,
                font: Font::Sansserif,
                font_size: fonts.meld_type_line,
                wrap_width: 440,
                ..Default::default()
            },

            meld_type_line_end_x: 20,

            adventure_type_line: TextStyle {
                x: 20,
                y: 310,
                font: Font::Sansserif,
                font_size: fonts.adventure_type_line,
                wrap_width: 372,
                ..Default::default()
            },

            meld_oracle: TextStyle {
                x: 20,
                y: 344,
                font: Font::Sansserif,
                font_size: fonts.meld_oracle,
                wrap_width: 500,
                ..Default::default()
            },

            rules: TextStyle {
                x: 20,
                y: 344,
                font: Font::Sansserif,
                font_size: fonts.rules,
                long_text_font_size: Some(fonts.small_rules),
                wrap_width: 372,
                max_length: Some(180),
                ..Default::default()
            },

            adventure_oracle_text_main_face: TextStyle {
                x: 224,
                y: 344,
                font: Font::Sansserif,
                font_size: fonts.rules,
                long_text_font_size: Some(fonts.small_rules),
                wrap_width: 180,
                ..Default::default()
            },

            adventure_oracle_text_alt_face: TextStyle {
                x: 20,
                y: 404,
                font: Font::Sansserif,
                font_size: fonts.rules,
                wrap_width: 186,
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
                y: 508,
                max_width: 40,
                max_height: 40,
            },

            planeswalker_loyalty_shield: SvgLayout {
                x: 325,
                y: 500,
                max_width: 90,
                max_height: 50,
            },

            meld_planeswalker_loyalty_shield: SvgLayout {
                x: 340,
                y: 20,
                max_width: 100,
                max_height: 150,
            },

            set_code: TextStyle {
                x: 30,
                y: 566,
                font: Font::Sansserif,
                font_size: fonts.set_code,
                wrap_width: 372,

                ..Default::default()
            },

            meld_set_code: TextStyle {
                // x: 380,
                margin_right: 20,
                y: 566,
                font: Font::Sansserif,
                font_size: fonts.set_code,
                wrap_width: 372,

                ..Default::default()
            },

            artist: TextStyle {
                x: 120,
                y: 566,
                font: Font::Sansserif,
                font_size: fonts.artist,
                long_text_font_size: Some(fonts.artist_small),
                wrap_width: 200,

                ..Default::default()
            },

            pow_tough_style: TextStyle {
                x: 330,
                y: 566,
                font: Font::Serif,
                font_size: fonts.pow_tough,

                wrap_width: 62,
                ..Default::default()
            },

            planeswalker_loyalty_style: TextStyle {
                x: 355,
                y: 540,
                font: Font::Serif,
                font_size: fonts.loyalty,
                wrap_width: 62,
                ..Default::default()
            },

            serif_font: fonts::MPLANTIN,
            sanserif_font: fonts::TAHOMA,
        }
    }
}

impl Layout {
    pub fn font_data(&self, font: Font) -> &[u8] {
        match font {
            Font::Serif => &self.serif_font,
            Font::Sansserif => &self.sanserif_font,
        }
    }

    pub fn text_width(&self, text: &str, style: &TextStyle) -> f32 {
        text_width(
            text,
            self.font_data(style.font),
            style.font_size,
            style.letter_spacing,
        )
    }
    pub fn wrapped_text_width(&self, text: &str, style: &TextStyle) -> f32 {
        wrapped_text_width(
            text,
            self.font_data(style.font),
            style.font_size,
            style.letter_spacing,
            style.wrap_width,
        )
    }

    pub fn wrapped_text_height(&self, text: &str, style: &TextStyle) -> i32 {
        wrapped_text_height(
            text,
            self.font_data(style.font),
            style.font_size,
            style.letter_spacing,
            style.wrap_width,
        )
    }
}
