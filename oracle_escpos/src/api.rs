use scryfall_oracle::{CardLayout, OracleScryfallCard};
use tokio::net::TcpStream;

use crate::{
    card::{card_type::CardType, image_gen::CardPrint},
    printer::print_img,
};

pub struct OraclePrinter {
    host: String,
    port: u16,
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
        card: OracleScryfallCard,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Spidey is a MDFC but we want to get that from the data itself
        let card_type = if card
            .core
            .type_line
            .as_deref()
            .is_some_and(|type_line| type_line.split_whitespace().any(|word| word == "Omen"))
        {
            CardType::Omen(card)
        } else {
            match card.core.layout {
                CardLayout::ModalDFC => CardType::MDFC(card),
                CardLayout::Adventure => CardType::Adventure(card),
                CardLayout::Prepare => CardType::Prepare(card),
                _ => CardType::Regular(card),
            }
        };

        let print = CardPrint::new(&card_type);
        let img = print.render(None).await?;
        let res = print_img(img, &self.host, self.port);

        match res {
            Ok(()) => tracing::info!("Successfully printed image"),
            Err(e) => tracing::error!(error = %e, "Failed to print image"),
        }

        Ok(())
    }

    // async fn print_scryfall_card(&self, card: ScryfallCard) {}
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
        let json = fs::read_to_string("./tests/miles_morales_test_card.json")?;
        let card: OracleScryfallCard = serde_json::from_str(&json)?;

        let printer = OraclePrinter::new("192.168.2.47".to_string(), 9100);

        printer.print_oracle_scryfall_card(card).await?;

        Ok(())
    }
}
