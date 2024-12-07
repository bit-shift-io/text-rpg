use bevy_ecs::world::World;
//use lazy_static::lazy_static;
use std::{collections::HashMap, io::Read, path::PathBuf, sync::Mutex};
use std::sync::LazyLock;

use crate::{config::Config, lib::aichat::AiChat};

// https://dev.to/leemeganj/how-to-use-the-lazy-initialization-pattern-with-rust-180-4n4k

/// Holds the config for the bot
pub static GLOBAL_CONFIG_2: LazyLock<Mutex<Option<Config>>> = LazyLock::new(|| { Mutex::new(None) });

/// Count of the global messages per user
pub static GLOBAL_MESSAGES_2: LazyLock<Mutex<HashMap<String, u64>>> = LazyLock::new(|| { Mutex::new(HashMap::new()) } );

// Bevy ECS contents
pub static GLOBAL_WORLD_2: LazyLock<Mutex<World>> = LazyLock::new(|| { Mutex::new(World::default()) });

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
pub fn get_ai_chat() -> AiChat {
    let config = GLOBAL_CONFIG_2.lock().unwrap().clone().unwrap();
    AiChat::new("aichat".to_string(), None) //Some("/Users/fabian/Library/Application Support/aichat/".to_owned())) //config.aichat_config_dir.clone())
}
