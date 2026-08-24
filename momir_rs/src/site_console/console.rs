use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use scryfall_oracle::OracleScryfallCard;
use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub enum ConsoleMessage {
    Text {
        sender: String,
        body: String,
    },
    Card {
        sender: String,
        card: OracleScryfallCard,
    },
}

#[derive(Clone)]
pub struct SiteConsole {
    games: Arc<RwLock<HashMap<String, broadcast::Sender<ConsoleMessage>>>>,
}
impl SiteConsole {
    pub fn new() -> Self {
        Self {
            games: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn sender(&self, game_id: &str) -> broadcast::Sender<ConsoleMessage> {
        let mut games = self.games.write().unwrap();

        games
            .entry(game_id.to_string())
            .or_insert_with(|| {
                let (tx, _) = broadcast::channel(100);
                tx
            })
            .clone()
    }

    pub fn send(&self, game_id: &str, message: ConsoleMessage) {
        let tx = self.sender(game_id);
        let _ = tx.send(message);
    }

    pub fn subscribe(&self, game_id: &str) -> broadcast::Receiver<ConsoleMessage> {
        self.sender(game_id).subscribe()
    }
}
