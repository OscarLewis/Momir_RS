use crate::OracleScryfallCard;

/// Helper methods for checking card information
impl OracleScryfallCard {
    /// Checks if the card's `type_line` contains a specific word
    pub fn has_type_word(&self, word: &str) -> bool {
        self.core
            .type_line
            .as_deref()
            .map_or(false, |line| line.split_whitespace().any(|w| w == word))
    }

    /// Convenience helper specifically for checking if the card is an Omen
    pub fn is_omen(&self) -> bool {
        self.has_type_word("Omen")
    }

    /// Convenience helper specifically for checking if the card is a Creature
    pub fn is_creature(&self) -> bool {
        self.has_type_word("Creature")
    }

    /// Convenience helper specifically for checking if the card is a Planeswalker
    pub fn is_planeswalker(&self) -> bool {
        self.has_type_word("Planeswalker")
    }
}
