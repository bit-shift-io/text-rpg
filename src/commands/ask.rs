use tracing::{error, info};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};
use regex::Regex;

use crate::{services::{command_context::CommandContext, extract::extract_between, game_info::GameInfo, prompt_builder::PromptBuilder}};
use crate::globals::*;


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

The player with name ${name} and has asked me the following question which you should respond to:
${question}.

${authority_block}
"#;

pub async fn ask(context: CommandContext) -> Result<(), ()> {
    let acting_player_member = context.sender_room_member().await?;
    let bot_name = context.bot_display_name().await.unwrap_or("Dungeon Master".to_string());

    let act_prompt = PromptBuilder::new(ASK_PROMPT)
        .bot_name(bot_name)
        .game_state(&context.game_info_as_json().await?)
        .build()
        .replace("${question}", &context.clean_text())
        .replace("${name}", acting_player_member.display_name().unwrap());

    let ask_response = context.execute_prompt(act_prompt).await?;

    context.room_send(&ask_response).await.unwrap();
    Ok(())
}