use tracing::{error, info};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};
use regex::Regex;

use crate::{services::{command_context::CommandContext, extract::{extract_between, extract_markdown_block}, game_info::GameInfo, prompt_builder::PromptBuilder}};
use crate::globals::*;


const ACT_PROMPT: &str = r#"
I am ${bot_name}, a dungeon master. I need you to take the current state of the game and update it to reflect a players action.

The player with name ${name} and has asked me to perform the following action (between action XML tags):
<action>
${action}
</action>

I need you to return a JSON object wrapped in a markdown code block with the language "json".
The current state of the game is:
```json
${game_state}
```

The game state is defined by the following TypeScript interface:
```typescript
${ts_definitions}
```

In the rules XML tag below I have included specific rules that must not be violated when changing the game state regardless of what the players action says:
<rules>
${rules}
The user may not invent items that are not in the game state.
If the user attacks a monster, the monster may retaliate and this should be reflected in the game state JSON.
You must return a valid JSON object satisfying the GameState interface.
Do not add additional fields, you may only modify existing fields.
Do not prompt for further instructions, if you are unsure make your best guess.
</rules>

${authority_block}

I also need a seperate short story placed between an opening xml tag <story> and the closing xml tag </story>.

In the story I need you to:
- Briefly describe for me change in JSON game state as a short, succinct story.
- Include exact values for things such as damage.

If the user is looking around, investigating or examining the room or area then include the following in the story:
- Concisely describe the room they are in, avoiding excessive detail.
- Concisely describe the entrances/exits to the room they are in that are not hidden (unless the player is specifically searching for hidden enterances).
- Concisely describe any items in the room they are in that are not hidden (unless the player is specifically searching for hidden items).
- Concisely describe any monsters in the room they are in that are not hidden (unless the player is specifically searching for hidden monsters).

If the player moves to another room then include the following in the story:
- Concisely describe the new room.
- Concisely describe monsters in the new room.

If any monsters performs an action after the player then include the following in the story:
- Briefly describe the monsters action.
- Include exact values for things such as damage.

If an objective has been met as a result of the action then include the following elements in the story:
- Concisely describe the objective met.
"#;

const OBJECTIVE_COMPLETED_PROMPT: &str = r#"
I am ${bot_name}, a dungeon master.
One or more objectives have been completed by the players!
The completed objective(s) are:
${completed_objectives}

The remaining objectives are:
${remaining_objectives}

The current game state is:
```json
${game_state}
```

I need you to return a response wrapped in <story> tags.
In the story:
- Congratulate the players.
- Provide a narrative description of the accomplishment.
- Clearly state what objectives remain (if any) and guide them on what is next in their journey.
"#;

pub async fn act(context: CommandContext) -> Result<(), ()> {
    let sender_player_member = context.sender_room_member().await?;
    let sender_display_name = sender_player_member.display_name().unwrap().to_owned();
    
    let sender_player_info = context.with_room_state(|state| {
        let game_info = state.game_info.as_ref().ok_or(())?;
        let f = game_info.player_characters.iter().find(|&player_character| player_character.name.eq(&sender_display_name));
        Ok(f.ok_or(())?.clone())
    }).await;

    let sender_player_info = match sender_player_info {
        Ok(info) => info,
        Err(_) => {
            // Player might not be in the game yet
            return Ok(());
        }
    };

    if !sender_player_info.is_alive() {
        context.room_send(&format!("🪦 {}", &sender_display_name)).await.unwrap();
        return Ok(());
    }
    
    let bot_name = context.bot_display_name().await.unwrap_or("Dungeon Master".to_string());

    // --- Round Logic Start ---
    let mut round_reset = false;
    let mut current_round_number = 0;
    
    let round_result = context.update_room_state(|state| {
        if let Some(round_info) = state.round_info.as_mut() {
            current_round_number = round_info.round_number;
            let has_acted = round_info.acted_players.contains(&sender_display_name);
            let elapsed = round_info.round_start_time.elapsed().unwrap_or(std::time::Duration::ZERO);
            
            // Get timeout from settings
            // We can't access GLOBAL_SETTINGS here easily inside the closure if we want to keep it simple.
            // Actually, we can just use a default or lock it.
            // Let's use 6 hours as default if not set.
            let timeout_duration = std::time::Duration::from_secs(6 * 60 * 60);
            let timeout = elapsed > timeout_duration;

            if has_acted {
                if timeout {
                    round_info.round_number += 1;
                    round_info.acted_players.clear();
                    round_info.round_start_time = std::time::SystemTime::now();
                    round_reset = true;
                    current_round_number = round_info.round_number;
                } else {
                    return Err(()); // Signal we need to show waiting message
                }
            }

            round_info.acted_players.push(sender_display_name.clone());
            Ok(())
        } else {
            Err(())
        }
    }).await;

    if round_result.is_err() {
        // Show waiting message
        let (round_num, hours, minutes, waiting_for) = context.with_room_state(|state| {
            let round_info = state.round_info.as_ref().ok_or(())?;
            let elapsed = round_info.round_start_time.elapsed().unwrap_or(std::time::Duration::ZERO);
            let timeout_duration = std::time::Duration::from_secs(6 * 60 * 60);
            let remaining = timeout_duration - elapsed;
            let remaining_secs = remaining.as_secs();
            
            let mut waiting = Vec::new();
            if let Some(game_info) = state.game_info.as_ref() {
                for player in &game_info.player_characters {
                    if player.is_alive() && !round_info.acted_players.contains(&player.name) {
                        waiting.push(player.name.clone());
                    }
                }
            }
            Ok((round_info.round_number, remaining_secs / 3600, (remaining_secs % 3600) / 60, waiting))
        }).await.unwrap_or((0, 0, 0, Vec::new()));

        let waiting_list = if waiting_for.is_empty() { "everyone".to_string() } else { waiting_for.join(", ") };
        context.room_send(&format!(
            "You have already acted in round {}. Please wait for: {}. Timeout in {}h {}m.", 
            round_num, waiting_list, hours, minutes
        )).await.unwrap();
        return Ok(());
    }

    if round_reset {
        context.room_send(&format!("Round timeout! Round {} started.", current_round_number)).await.unwrap();
    }
    // --- Round Logic End ---

    let act_prompt = PromptBuilder::new(ACT_PROMPT)
        .bot_name(bot_name)
        .game_state(&context.game_info_as_json().await?)
        .with_rules("") 
        .build()
        .replace("${action}", &context.clean_text())
        .replace("${name}", &sender_display_name);

    let act_response = match context.execute_prompt(act_prompt).await {
        Ok(res) => res,
        Err(_) => {
             // Revert turn on failure
             context.update_room_state(|state| {
                 if let Some(round_info) = state.round_info.as_mut() {
                     if let Some(pos) = round_info.acted_players.iter().position(|x| *x == sender_display_name) {
                         round_info.acted_players.remove(pos);
                     }
                 }
                 Ok(())
             }).await.ok();
             return Err(());
        }
    };

    let json_strs = extract_markdown_block("json", &act_response)?;
    if json_strs.len() == 0 {
        context.room_send("[act] Failed to get JSON from response.").await.unwrap();
        context.update_room_state(|state| {
             if let Some(round_info) = state.round_info.as_mut() {
                 if let Some(pos) = round_info.acted_players.iter().position(|x| *x == sender_display_name) {
                     round_info.acted_players.remove(pos);
                 }
             }
             Ok(())
        }).await.ok();
        return Ok(());
    }

    let story_strs = extract_between("<story>", "</story>", &act_response)?;
    if story_strs.len() == 0 {
        context.room_send("[act] Failed to get story block from response.").await.unwrap();
        context.update_room_state(|state| {
             if let Some(round_info) = state.round_info.as_mut() {
                 if let Some(pos) = round_info.acted_players.iter().position(|x| *x == sender_display_name) {
                     round_info.acted_players.remove(pos);
                 }
             }
             Ok(())
        }).await.ok();
        return Ok(());
    }

    let new_game_info = match GameInfo::from_str(&json_strs[0]) {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to parse JSON: {}", &json_strs[0]);
            context.room_send("Failed to parse JSON.").await.unwrap();
            context.update_room_state(|state| {
                 if let Some(round_info) = state.round_info.as_mut() {
                     if let Some(pos) = round_info.acted_players.iter().position(|x| *x == sender_display_name) {
                         round_info.acted_players.remove(pos);
                     }
                 }
                 Ok(())
            }).await.ok();
            return Ok(());
        }
    };
    
    let total_alive_players = new_game_info.player_characters.iter().filter(|p| p.is_alive()).count();

    // Capture old game info for objective comparison
    let old_info = context.with_room_state(|state| Ok(state.game_info.clone())).await.unwrap_or(None);
    
    context.update_room_state(|state| {
        state.game_info = Some(new_game_info.clone());
        Ok(())
    }).await?;

    context.room_send(&story_strs[0]).await.unwrap();

    if let Some(old_info) = old_info {
        check_objectives(&context, &old_info, &new_game_info).await.ok();
    }

    // --- Check End of Round ---
    context.update_room_state(|state| {
        if let Some(round_info) = state.round_info.as_mut() {
             if round_info.acted_players.len() >= total_alive_players {
                  round_info.round_number += 1;
                  round_info.acted_players.clear();
                  round_info.round_start_time = std::time::SystemTime::now();
                  // We can't send message inside the closure easily, we return a signal.
                  return Ok(true);
             }
        }
        Ok(false)
    }).await.map(|completed| {
        if completed {
             // Fetch round number to announce
             // This is a bit awkward but keeps the closure pure.
             // Actually, we could just return the round number.
        }
    }).ok();

    // Let's re-fetch round number for announcement if needed.
    let round_announce = context.with_room_state(|state| {
        if let Some(ri) = state.round_info.as_ref() {
            if ri.acted_players.is_empty() {
                return Ok(Some(ri.round_number));
            }
        }
        Ok(None)
    }).await.unwrap_or(None);

    if let Some(rn) = round_announce {
         context.room_send(&format!("All players acted! Round {} started.", rn)).await.unwrap();
    }

    Ok(())
}

async fn check_objectives(context: &CommandContext, old_info: &GameInfo, new_info: &GameInfo) -> Result<(), ()> {
    let mut completed_now = Vec::new();
    for new_obj in &new_info.objectives {
        if new_obj.completed {
            // Check if it was already completed in old_info
            // We match by goal string as we don't have IDs
            let was_completed = old_info.objectives.iter().any(|old_obj| old_obj.goal == new_obj.goal && old_obj.completed);
            if !was_completed {
                completed_now.push(new_obj.clone());
            }
        }
    }

    if completed_now.is_empty() {
        return Ok(());
    }

    let all_completed = new_info.objectives.iter().all(|o| o.completed);

    if all_completed {
        // Trigger end game logic
        return crate::commands::end::execute_end(context).await;
    }

    // Trigger objective completed message
    let bot_name = context.bot_display_name().await.unwrap_or("Dungeon Master".to_string());
    
    let completed_list = completed_now.iter().map(|o| format!("- {}", o.goal)).collect::<Vec<_>>().join("\n");
    let remaining = new_info.objectives.iter().filter(|o| !o.completed).map(|o| format!("- {}", o.goal)).collect::<Vec<_>>().join("\n");
    let remaining_list = if remaining.is_empty() { "None".to_string() } else { remaining };

    let prompt = PromptBuilder::new(OBJECTIVE_COMPLETED_PROMPT)
         .bot_name(bot_name)
         .build()
         .replace("${completed_objectives}", &completed_list)
         .replace("${remaining_objectives}", &remaining_list)
         .replace("${game_state}", &new_info.to_json_string().unwrap_or_default());
    
    let response = context.execute_prompt(prompt).await.unwrap_or_default();
    if let Ok(stories) = extract_between("<story>", "</story>", &response) {
         if !stories.is_empty() {
             context.room_send(&stories[0]).await.unwrap();
         }
    }
    
    Ok(())
}