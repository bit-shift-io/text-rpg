use tracing::{error, info};
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::{get_ai_chat, services::command_context::CommandContext};
use crate::globals::*;

pub async fn dump(context: CommandContext) -> Result<(), ()> {
    let game_info_json: String = context.with_game_info(|game_info| {
       game_info.to_json_string()
    }).await?;
    context.room_send(&game_info_json).await?;
    Ok(())
}
