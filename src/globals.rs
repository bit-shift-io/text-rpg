use tracing::info;
use std::fs;
use std::{collections::HashMap, io::Read, path::PathBuf};
use std::sync::LazyLock;

use crate::llm_client::LlmClient;
use crate::services::game_info::GameInfo;
use crate::{config::Config};

// https://stackoverflow.com/questions/68976937/rust-future-cannot-be-sent-between-threads-safely
use tokio::sync::Mutex;

// https://dev.to/leemeganj/how-to-use-the-lazy-initialization-pattern-with-rust-180-4n4k

/// Holds the config for the bot
pub static GLOBAL_CONFIG: LazyLock<Mutex<Option<Config>>> = LazyLock::new(|| { Mutex::new(None) });

/// Holds the LLM client
pub static GLOBAL_LLM_CLIENT: LazyLock<Mutex<Option<LlmClient>>> = LazyLock::new(|| { Mutex::new(None) });


// Holds the current game info
pub static GLOBAL_GAME_INFO: LazyLock<Mutex<Option<GameInfo>>> = LazyLock::new(|| { Mutex::new(None) });


#[derive(Clone, Debug)]
pub struct RoundInfo {
    pub round_number: u32,
    pub acted_players: Vec<String>,
    pub round_start_time: std::time::SystemTime,
}

pub static GLOBAL_ROUND_INFO: LazyLock<Mutex<Option<RoundInfo>>> = LazyLock::new(|| { Mutex::new(None) });

// Holds the list of joined players (User IDs)
pub static GLOBAL_LOBBY: LazyLock<Mutex<Vec<String>>> = LazyLock::new(|| { Mutex::new(Vec::new()) });

use crate::settings::Settings;
pub static GLOBAL_SETTINGS: LazyLock<Mutex<Settings>> = LazyLock::new(|| { Mutex::new(Settings::load()) });
