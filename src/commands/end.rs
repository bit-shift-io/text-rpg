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
    let game_state_json = match context.game_info_as_json().await {
        Ok(json) => json,
        Err(_) => {
            // Already handled by game_info_as_json if it sends error message, 
            // but let's be explicit if needed.
            return Ok(());
        }
    };

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
    context.update_room_state(|state| {
        state.game_info = None;
        state.round_info = None;
        Ok(())
    }).await?;

    context.room_send("The game has ended. You may start a new one with .start").await.unwrap();

    Ok(())
}
