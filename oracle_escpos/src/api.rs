use core::fmt;
use image::ImageBuffer;
use momir_oracle_config::AppConfig;
use scryfall_oracle::{CardLayout, OracleScryfallCard};
use std::path::PathBuf;
use tokio::net::TcpStream;

use crate::{
    card::{card_type::CardType, image_gen::CardPrint},
    printer::{print_img_network, print_img_usb},
};

#[derive(Debug)]
pub enum PrinterError {
    Render(String),
    Print(String),
}

impl fmt::Display for PrinterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Render(err) => write!(f, "Failed to render card: {err}"),
            Self::Print(err) => write!(f, "Failed to print card: {err}"),
        }
    }
}

impl std::error::Error for PrinterError {}

#[derive(Clone)]
pub struct OracleNetworkPrinter {
    host: String,
    port: u16,
}

pub struct OracleUsbPrinter {
    path: PathBuf,
}

impl OracleNetworkPrinter {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub fn from_config(config: &AppConfig) -> Option<Self> {
        Some(Self {
            host: config.printer.host.clone()?,
            port: config.printer.port?,
        })
    }

    pub async fn check_connection(&self) -> bool {
        TcpStream::connect((&*self.host, self.port)).await.is_ok()
    }

    pub async fn print_oracle_scryfall_card(
        &self,
        card: &OracleScryfallCard,
        image_out_path: Option<&PathBuf>,
    ) -> Result<(), PrinterError> {
        let img = render_card(card).await?;

        print_img_network(img, &self.host, self.port)
            .map_err(|e| PrinterError::Print(e.to_string()))?;

        Ok(())
    }
}

impl OracleUsbPrinter {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn from_config(config: &AppConfig) -> Option<Self> {
        Some(Self {
            path: PathBuf::from(config.printer.usb_path.clone()?),
        })
    }

    pub fn check_connection(&self) -> bool {
        self.path.exists()
    }

    pub async fn print_oracle_scryfall_card(
        &self,
        card: &OracleScryfallCard,
        image_out_path: Option<&PathBuf>,
    ) -> Result<(), PrinterError> {
        let img = render_card(card).await?;

        print_img_usb(img, &self.path).map_err(|e| PrinterError::Print(e.to_string()))?;

        Ok(())
    }
}

async fn render_card(
    card: &OracleScryfallCard,
) -> Result<ImageBuffer<image::Rgb<u8>, Vec<u8>>, PrinterError> {
    let card_type = if card
        .core
        .type_line
        .as_deref()
        .is_some_and(|type_line| type_line.split_whitespace().any(|word| word == "Omen"))
    {
        CardType::Omen(card.clone())
    } else {
        match card.core.layout {
            CardLayout::ModalDFC => CardType::MDFC(card.clone()),
            CardLayout::Adventure => CardType::Adventure(card.clone()),
            CardLayout::Prepare => CardType::Prepare(card.clone()),
            CardLayout::Transform => CardType::MDFC(card.clone()),
            CardLayout::Split => CardType::MDFC(card.clone()),
            _ => CardType::Regular(card.clone()),
        }
    };

    CardPrint::new(&card_type)
        .render(None)
        .await
        .map_err(|e| PrinterError::Render(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use scryfall_oracle::OracleScryfallCard;
    use std::fs;
    use test_log::test;

    #[test(tokio::test)]
    #[ignore = "requires network printer hardware"]
    async fn test_integration_print() -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string("./tests/karolina_test_card.json")?;
        let card: OracleScryfallCard = serde_json::from_str(&json)?;

        let printer = OracleNetworkPrinter::new("192.168.2.47".to_string(), 9100);

        printer.print_oracle_scryfall_card(&card, None).await?;

        Ok(())
    }
}
