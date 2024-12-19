use tracing::info;
use std::fs;
use std::{collections::HashMap, io::Read, path::PathBuf};
use std::sync::LazyLock;

use crate::services::game_info::GameInfo;
use crate::{config::Config, services::aichat::AiChat};

// https://stackoverflow.com/questions/68976937/rust-future-cannot-be-sent-between-threads-safely
use tokio::sync::Mutex;

// https://dev.to/leemeganj/how-to-use-the-lazy-initialization-pattern-with-rust-180-4n4k

/// Holds the config for the bot
pub static GLOBAL_CONFIG: LazyLock<Mutex<Option<Config>>> = LazyLock::new(|| { Mutex::new(None) });

// Holds the current game info
pub static GLOBAL_GAME_INFO: LazyLock<Mutex<Option<GameInfo>>> = LazyLock::new(|| { Mutex::new(None) });


/// Returns the backend based on the global config
pub async fn get_ai_chat() -> AiChat {
    let config = GLOBAL_CONFIG.lock().await.clone().unwrap();
    AiChat::new("aichat".to_string(), config.aichat_config_file) //Some("/Users/fabian/Library/Application Support/aichat/".to_owned())) //config.aichat_config_dir.clone())
}
