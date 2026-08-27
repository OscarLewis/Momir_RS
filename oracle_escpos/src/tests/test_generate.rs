#[cfg(test)]
mod tests {
    use crate::CardImage;

    use scryfall_oracle::OracleScryfallCard;
    use std::{fs, path::PathBuf};
    use test_log::test;

    /// Generates a card image from the test card JSON
    #[test(tokio::test)]
    #[ignore]
    async fn test_generate() -> Result<(), Box<dyn std::error::Error>> {
        let path = "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/tests/sokka_suki_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from(
                "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/card.png",
            ))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_long_name() -> Result<(), Box<dyn std::error::Error>> {
        let path = "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/tests/asmoranomardicadaistinaculdacar_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from(
                "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/asmoranomardicadaistinaculdacar_card.png",
            ))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_very_long_name() -> Result<(), Box<dyn std::error::Error>> {
        let path = "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/tests/customer_service_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from(
                "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/customer_service_card.png",
            ))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_extremely_long_name() -> Result<(), Box<dyn std::error::Error>> {
        let path = "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/tests/market_research_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from(
                "/home/oscar/Documents/Projects/momir_rs_workspace/oracle_escpos/renders/market_research_card.png",
            ))
            .await?;

        Ok(())
    }
}
