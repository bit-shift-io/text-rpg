use serde::{Serialize, Deserialize};
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

// Holds the LLM client
pub static GLOBAL_LLM_CLIENT: LazyLock<Mutex<Option<LlmClient>>> = LazyLock::new(|| { Mutex::new(None) });


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RoundInfo {
    pub round_number: u32,
    pub acted_players: Vec<String>,
    pub round_start_time: std::time::SystemTime,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct RoomState {
    pub room_id: String,
    pub game_info: Option<GameInfo>,
    pub round_info: Option<RoundInfo>,
    pub lobby: Vec<String>, // User IDs
}

/// Holds the states for all rooms
pub static GLOBAL_ROOM_STATES: LazyLock<Mutex<HashMap<String, RoomState>>> = LazyLock::new(|| { Mutex::new(HashMap::new()) });

pub async fn save_room_state(room_id: &str, state: &RoomState) {
    let path = format!("data/room_{}.json", room_id.replace("!", "").replace(":", "_"));
    match serde_json::to_string_pretty(state) {
        Ok(json) => {
            if let Err(e) = fs::write(&path, json) {
                tracing::error!("Failed to save room state to {}: {}", path, e);
            }
        },
        Err(e) => {
            tracing::error!("Failed to serialize room state for {}: {}", room_id, e);
        }
    }
}

pub async fn load_all_room_states() {
    info!("Loading all room states from data/ directory...");
    let entries = match fs::read_dir("data") {
        Ok(entries) => entries,
        Err(e) => {
            tracing::warn!("Could not read data directory: {}", e);
            return;
        }
    };

    let mut states_guard = GLOBAL_ROOM_STATES.lock().await;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            let file_name = path.file_name().unwrap().to_string_lossy();
            if file_name.starts_with("room_") && file_name != "settings.json" {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        match serde_json::from_str::<RoomState>(&content) {
                            Ok(state) => {
                                info!("Restored state for room: {}", state.room_id);
                                states_guard.insert(state.room_id.clone(), state);
                            },
                            Err(e) => tracing::error!("Failed to parse {}: {}", path.display(), e),
                        }
                    },
                    Err(e) => tracing::error!("Failed to read {}: {}", path.display(), e),
                }
            }
        }
    }
}

use crate::settings::Settings;
pub static GLOBAL_SETTINGS: LazyLock<Mutex<Settings>> = LazyLock::new(|| { Mutex::new(Settings::load()) });
