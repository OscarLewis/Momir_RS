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
        let path = "./tests/sokka_suki_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image.generate(&PathBuf::from("./renders/card.png")).await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_lucy_in_the_sky() -> Result<(), Box<dyn std::error::Error>> {
        let path = "./tests/karolina_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from("./renders/karolina_card.png"))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_janeway() -> Result<(), Box<dyn std::error::Error>> {
        let path = "./tests/janeway_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from("./renders/janeway_card.png"))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_spidey() -> Result<(), Box<dyn std::error::Error>> {
        let path = "./tests/miles_morales_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from("./renders/miles_morales_card.png"))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_zeta_mulldrifter() -> Result<(), Box<dyn std::error::Error>> {
        let path = "./tests/zeta_mull_drifter_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from("./renders/zeta_mull_drifter_card.png"))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_bmf() -> Result<(), Box<dyn std::error::Error>> {
        let path = "./tests/bfm_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from("./renders/bfm_card.png"))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_long_name() -> Result<(), Box<dyn std::error::Error>> {
        let path = "./tests/asmoranomardicadaistinaculdacar_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from(
                "./renders/asmoranomardicadaistinaculdacar_card.png",
            ))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_very_long_name() -> Result<(), Box<dyn std::error::Error>> {
        let path = "./tests/customer_service_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from("./renders/customer_service_card.png"))
            .await?;

        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_extremely_long_name() -> Result<(), Box<dyn std::error::Error>> {
        let path = "./tests/market_research_test_card.json";

        let json = fs::read_to_string(path).expect("failed to read test card");

        let card: OracleScryfallCard =
            serde_json::from_str(&json).expect("failed to deserialize test card");

        let image = CardImage::new(card);

        image
            .generate(&PathBuf::from("./renders/market_research_card.png"))
            .await?;

        Ok(())
    }
}
