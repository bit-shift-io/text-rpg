use tracing::{error, info};
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::{get_ai_chat, services::command_context::CommandContext};
use crate::globals::*;

pub async fn dump(context: CommandContext) -> Result<(), ()> {
    let game_info = context.clone_game_info().await?;
    let game_info_json = serde_json::to_string_pretty(&game_info).unwrap();
    context.room_send(&game_info_json).await.unwrap();
    Ok(())
}
