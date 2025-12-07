use tracing::{error, info};
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::{services::command_context::CommandContext};
use crate::globals::*;

pub async fn dump(context: CommandContext) -> Result<(), ()> {
    context.room_send(&context.game_info_as_json().await?).await?;
    Ok(())
}
