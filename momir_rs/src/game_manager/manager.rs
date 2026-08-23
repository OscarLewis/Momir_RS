use std::{
    collections::HashSet,
    sync::{Arc, RwLock},
};

use uuid::Uuid;

#[derive(Clone)]
pub struct GameManager {
    games: Arc<RwLock<HashSet<String>>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    pub fn new_game(&self) -> String {
        let game_id = Uuid::new_v4().to_string();

        self.games.write().unwrap().insert(game_id.clone());

        game_id
    }

    pub fn exists(&self, game_id: &str) -> bool {
        self.games.read().unwrap().contains(game_id)
    }
}
