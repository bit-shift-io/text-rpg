//use bevy_ecs::world::World;
use tracing::info;
use std::fs;
//use lazy_static::lazy_static;
use std::{collections::HashMap, io::Read, path::PathBuf};
use std::sync::LazyLock;

use crate::components::game_info_container::GameInfo;
use crate::{config::Config, lib::aichat::AiChat};

// https://stackoverflow.com/questions/68976937/rust-future-cannot-be-sent-between-threads-safely
use tokio::sync::Mutex;

// https://dev.to/leemeganj/how-to-use-the-lazy-initialization-pattern-with-rust-180-4n4k

/// Holds the config for the bot
pub static GLOBAL_CONFIG: LazyLock<Mutex<Option<Config>>> = LazyLock::new(|| { Mutex::new(None) });

/// Count of the global messages per user
pub static GLOBAL_MESSAGES: LazyLock<Mutex<HashMap<String, u64>>> = LazyLock::new(|| { Mutex::new(HashMap::new()) } );

// Bevy ECS contents
//pub static GLOBAL_WORLD_2: LazyLock<Mutex<World>> = LazyLock::new(|| { Mutex::new(World::default()) });

// Holds the current game info
pub static GLOBAL_GAME_INFO: LazyLock<Mutex<Option<GameInfo>>> = LazyLock::new(|| { Mutex::new(None) });

/*
lazy_static! {
    /// Holds the config for the bot
    pub static ref GLOBAL_CONFIG: Mutex<Option<Config>> = Mutex::new(None);

    /// Count of the global messages per user
    pub static ref GLOBAL_MESSAGES: Mutex<HashMap<String, u64>> = Mutex::new(HashMap::new());

    // Bevy ECS contents
    pub static ref GLOBAL_WORLD: Mutex<World> = Mutex::new(World::default());
}*/




/// Returns the backend based on the global config
pub async fn get_ai_chat() -> AiChat {
    let config = GLOBAL_CONFIG.lock().await.clone().unwrap();
    AiChat::new("aichat".to_string(), config.aichat_config_file) //Some("/Users/fabian/Library/Application Support/aichat/".to_owned())) //config.aichat_config_dir.clone())
}
