#[derive(Debug, Clone)]
pub struct TextStyle {
    pub x: i32,
    pub y: i32,
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
            long_text_font_size: None,
            letter_spacing: 0.0,
            wrap_width: 372,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Layout {
    pub width: u32,
    pub height: u32,

    pub art: ArtLayout,

    pub name: TextStyle,
    pub type_line: TextStyle,
    pub rules: TextStyle,
    pub flavor: TextStyle,
    pub metadata: TextStyle,
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
        Self {
            width: 412,
            height: 576,

            art: ArtLayout {
                x: 60,
                y: 80,
                max_width: 300,
                max_height: 200,
            },

            name: TextStyle {
                x: 20,
                y: 35,
                font_size: 34.0,
                long_text_font_size: Some(24.0),
                letter_spacing: 0.0,
                wrap_width: 372,
            },

            type_line: TextStyle {
                x: 20,
                y: 310,
                font_size: 22.0,
                wrap_width: 372,
                ..Default::default()
            },

            rules: TextStyle {
                x: 20,
                y: 340,
                font_size: 19.0,
                letter_spacing: 1.0,
                wrap_width: 372,
                ..Default::default()
            },

            flavor: TextStyle {
                x: 20,
                y: 0,
                font_size: 17.0,
                wrap_width: 372,
                ..Default::default()
            },

            metadata: TextStyle {
                x: 20,
                y: 0,
                font_size: 14.0,
                wrap_width: 372,
                ..Default::default()
            },
        }
    }
}
