use tracing::{error, info};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};
use regex::Regex;

use crate::{get_ai_chat, services::{command_context::CommandContext, extract::extract_between, game_info::GameInfo}};
use crate::globals::*;


const ASK_PROMPT: &str = r#"
I am a dungeon master.

The current state of the game in JSON format is:
<json>
${game_state}
</json>

The player with name ${name} and has asked me the following question which you should respond to:
${question}.
"#;

pub async fn ask(context: CommandContext) -> Result<(), ()> {
    let acting_player_member = context.sender_room_member().await?;

    let act_prompt = ASK_PROMPT
        .replace("${game_state}", &context.game_info_as_json().await?)
        .replace("${question}", &context.clean_text())
        .replace("${name}", acting_player_member.display_name().unwrap());

    let ask_response = context.execute_prompt(act_prompt).await?;

    context.room_send(&ask_response).await.unwrap();
    Ok(())
}