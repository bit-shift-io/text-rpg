use tracing::{error, info};
use crate::{services::{command_context::CommandContext, extract::extract_between, prompt_builder::PromptBuilder}, globals::*};

const END_PROMPT: &str = r#"
I am ${bot_name}, a dungeon master.
The game is ending.

The current game state is:
```json
${game_state}
```

I need you to generate a concluding story for the adventure.
Wrap the story in <story> XML tags.
The story should:
- Acknowledge the end of the journey.
- Summarize the achievements based on completed objectives.
- Provide a satisfying conclusion for the characters.
"#;

pub async fn end(context: CommandContext) -> Result<(), ()> {
    execute_end(&context).await
}

pub async fn execute_end(context: &CommandContext) -> Result<(), ()> {
    let bot_name = context.bot_display_name().await.unwrap_or("Dungeon Master".to_string());
    
    // Check if game exists
    let game_info_guard = GLOBAL_GAME_INFO.lock().await;
    let game_state_json = if let Some(game_info) = game_info_guard.as_ref() {
        game_info.to_json_string().unwrap_or_default()
    } else {
        context.room_send("No active game to end.").await.unwrap();
        return Ok(());
    };
    drop(game_info_guard); // Release lock before long await

    let end_prompt = PromptBuilder::new(END_PROMPT)
        .bot_name(bot_name)
        .build()
        .replace("${game_state}", &game_state_json);

    let response = match context.execute_prompt(end_prompt).await {
        Ok(res) => res,
        Err(_) => {
            context.room_send("Failed to generate ending story.").await.unwrap();
            return Ok(());
        }
    };

    let story_strs = match extract_between("<story>", "</story>", &response) {
        Ok(s) => s,
        Err(_) => Vec::new()
    };

    if story_strs.is_empty() {
        context.room_send("[end] Failed to parse ending story.").await.unwrap();
    } else {
        context.room_send(&story_strs[0]).await.unwrap();
    }

    // Cleanup
    {
        let mut game_info = GLOBAL_GAME_INFO.lock().await;
        *game_info = None;
    }
    {
        let mut round_info = GLOBAL_ROUND_INFO.lock().await;
        *round_info = None;
    }

    context.room_send("The game has ended. You may start a new one with .start").await.unwrap();

    Ok(())
}
