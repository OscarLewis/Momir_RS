use core::fmt;
use std::path::PathBuf;

use momir_oracle_config::AppConfig;
use scryfall_oracle::{CardLayout, OracleScryfallCard};
use tokio::net::TcpStream;
use tracing::debug;

use crate::{
    card::{card_type::CardType, image_gen::CardPrint},
    printer::print_img,
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
pub struct OraclePrinter {
    host: String,
    port: u16,
}

impl From<&AppConfig> for OraclePrinter {
    fn from(config: &AppConfig) -> Self {
        Self {
            host: config.printer.host.clone(),
            port: config.printer.port,
        }
    }
}

impl OraclePrinter {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }

    pub async fn check_connection(&self) -> bool {
        TcpStream::connect((&*self.host, self.port)).await.is_ok()
    }

    pub async fn print_oracle_scryfall_card(
        &self,
        card: &OracleScryfallCard,
        image_out_path: Option<&PathBuf>,
    ) -> Result<(), PrinterError> {
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

        let print = CardPrint::new(&card_type);

        let img = print
            .render(None)
            .await
            .map_err(|e| PrinterError::Render(e.to_string()))?;

        print_img(img, &self.host, self.port).map_err(|e| PrinterError::Print(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scryfall_oracle::OracleScryfallCard;
    use std::fs;
    use test_log::test;

    #[test(tokio::test)]
    #[ignore = "requires network printer hardware at 192.168.2.47"]
    async fn test_integration_print() -> Result<(), Box<dyn std::error::Error>> {
        let json = fs::read_to_string("./tests/karolina_test_card.json")?;
        let card: OracleScryfallCard = serde_json::from_str(&json)?;

        let printer = OraclePrinter::new("192.168.2.47".to_string(), 9100);

        printer.print_oracle_scryfall_card(&card, None).await?;

        Ok(())
    }
}
