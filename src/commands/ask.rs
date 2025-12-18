use tracing::{error, info};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};
use regex::Regex;

use crate::{services::{command_context::CommandContext, extract::extract_between, game_info::GameInfo, prompt_builder::PromptBuilder}};
use crate::globals::*;


use std::time::Duration;

const ASK_PROMPT: &str = r#"
I am ${bot_name}, a dungeon master.

The current state of the game in JSON format is:
```json
${game_state}
```

The game state is defined by the following TypeScript interface:
```typescript
${ts_definitions}
```

${round_info}

The player with name ${name} and has asked me the following question which you should respond to:
${question}.

${authority_block}
"#;

pub async fn ask(context: CommandContext) -> Result<(), ()> {
    let acting_player_member = context.sender_room_member().await?;
    let bot_name = context.bot_display_name().await.unwrap_or("Dungeon Master".to_string());

    let round_info_str = context.with_room_state(|state| {
        if let Some(round_info) = state.round_info.as_ref() {
             let now = std::time::SystemTime::now();
             let elapsed = now.duration_since(round_info.round_start_time).unwrap_or_default();
             
             let timeout_duration = std::time::Duration::from_secs(6 * 60 * 60); 
             
             let remaining = if elapsed < timeout_duration { timeout_duration - elapsed } else { std::time::Duration::ZERO };
             let remaining_secs = remaining.as_secs();
             let hours = remaining_secs / 3600;
             let minutes = (remaining_secs % 3600) / 60;
             
             let acted_list = if round_info.acted_players.is_empty() {
                 "None".to_string()
             } else {
                 round_info.acted_players.join(", ")
             };

             Ok(format!(
                "Current Round Information:\n- Round Number: {}\n- Players who have acted this round: {}\n- Time remaining in round: {}h {}m",
                round_info.round_number,
                acted_list,
                hours,
                minutes
             ))
        } else {
            Ok("Round Information: Not available (Game might not be started)".to_string())
        }
    }).await.unwrap_or_else(|_| "Round Information: Not available (Game might not be started)".to_string());

    let act_prompt = PromptBuilder::new(ASK_PROMPT)
        .bot_name(bot_name)
        .game_state(&context.game_info_as_json().await?)
        .build()
        .replace("${question}", &context.clean_text())
        .replace("${name}", acting_player_member.display_name().unwrap())
        .replace("${round_info}", &round_info_str);

    let ask_response = context.execute_prompt(act_prompt).await?;

    context.room_send(&ask_response).await.unwrap();
    Ok(())
}