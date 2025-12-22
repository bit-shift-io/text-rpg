use crate::services::game_info::GameInfo;
use crate::globals::*;
use crate::services::prompt_builder::PromptBuilder;
use tracing::{info, error};

const SUMMARY_PROMPT: &str = r#"
I am ${bot_name}, a dungeon master.
The bot has just restarted, and I need a brief "story so far" summary to catch the players up.

The current game state in JSON format is:
```json
${game_state}
```

Please provide a short, 1-2 paragraph summary that includes:
- Where the party is currently.
- What they have achieved so far (completed objectives).
- What they are currently trying to do.

Be atmospheric but concise.
Wrap the summary in <story> XML tags.
"#;

pub async fn generate_story_summary(game_info: &GameInfo, bot_name: &str) -> Option<String> {
    let prompt = PromptBuilder::new(SUMMARY_PROMPT)
        .bot_name(bot_name.to_string())
        .build()
        .replace("${game_state}", &game_info.to_json_string().unwrap_or_default());

    let mut client_guard = GLOBAL_LLM_CLIENT.lock().await;
    if let Some(client) = client_guard.as_mut() {
        info!("Generating story summary for resumption...");
        match client.chat(&prompt).await {
            Ok(response) => {
                if let Ok(stories) = crate::services::extract::extract_between("<story>", "</story>", &response) {
                    if !stories.is_empty() {
                        return Some(stories[0].clone());
                    }
                }
                Some(response)
            },
            Err(e) => {
                error!("Failed to generate story summary: {}", e);
                None
            }
        }
    } else {
        None
    }
}
