#[derive(Debug, Clone, Copy)]
pub enum Font {
    Serif,
    Sansserif,
}
pub mod fonts {
    pub static MPLANTIN: &[u8] = include_bytes!("../../static/fonts/Mplantin.ttf");
    pub static TAHOMA: &[u8] = include_bytes!("../../static/fonts/tahoma.ttf");
}

pub fn get_font(font: Font) -> &'static [u8] {
    match font {
        Font::Serif => fonts::MPLANTIN,
        Font::Sansserif => fonts::TAHOMA,
    }
}
