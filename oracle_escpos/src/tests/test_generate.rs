#[cfg(test)]
mod tests {
    use crate::card::card_type::CardType;
    use crate::card::image_gen::CardPrint;
    use scryfall_oracle::{CardLayout, OracleScryfallCard};
    use std::{fs, path::PathBuf};
    use test_log::test;

    fn load_card(path: &str) -> Result<OracleScryfallCard, Box<dyn std::error::Error>> {
        let json = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }

    /// Generates a card image from the test card JSON
    #[test(tokio::test)]
    #[ignore]
    async fn test_generate() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/sokka_suki_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_spidey() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/miles_morales_test_card.json")?;

        // Spidey is a MDFC but we want to get that from the data itself
        let card_type = match card.core.layout {
            CardLayout::ModalDFC => CardType::MDFC(card),
            _ => CardType::Regular(card),
        };

        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/miles_morales_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_etali() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/etali_test_card.json")?;

        let card_type = match card.core.layout {
            CardLayout::Transform => CardType::MDFC(card),
            _ => CardType::Regular(card),
        };

        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/etali_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_tormentor() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/tomentor_mist_transform_test_card.json")?;

        let card_type = match card.core.layout {
            CardLayout::Transform => CardType::MDFC(card),
            _ => CardType::Regular(card),
        };

        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/tomentor_mist_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_boom_bust() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/boom_bust_card.json")?;

        let card_type = match card.core.layout {
            CardLayout::Split => CardType::MDFC(card),
            _ => CardType::Regular(card),
        };

        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/boom_bust_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_lucy_in_the_sky() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/karolina_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/karolina_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_chronologist_level_up() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/chronologist_level_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/chronologist_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_janeway() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/janeway_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/janeway_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_elspeth_plansewalker() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/eslpeth_planeswalker_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/eslpeth_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_zeta_mulldrifter() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/zeta_mull_drifter_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/zeta_mull_drifter_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_kaelin_adventure() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/kaelin_test_card.json")?;
        let card_type = CardType::Adventure(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/kaelin_adventure_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_beluna_adventure() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/beluna_test_card.json")?;
        let card_type = CardType::Adventure(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/beluna_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_bear_prepare() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/bear_prepare_test_card.json")?;
        let card_type = CardType::Prepare(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/bear_prepare_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_wilding_omen() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/wilding_omen_test_card.json")?;
        let card_type = CardType::Omen(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/wilding_omen_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_bmf() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/bfm_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/bfm_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_meld() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/urza_meld_test_card.json")?;
        let card_type = CardType::Meld(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/urza_meld_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_long_name() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/asmoranomardicadaistinaculdacar_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from(
                "./renders/asmoranomardicadaistinaculdacar_card.png",
            )))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_very_long_name() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/customer_service_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/customer_service_card.png")))
            .await?;
        Ok(())
    }

    #[test(tokio::test)]
    #[ignore]
    async fn test_extremely_long_name() -> Result<(), Box<dyn std::error::Error>> {
        let card = load_card("./tests/market_research_test_card.json")?;
        let card_type = CardType::Regular(card);
        let print = CardPrint::new(&card_type);
        print
            .render(Some(&PathBuf::from("./renders/market_research_card.png")))
            .await?;
        Ok(())
    }
}
