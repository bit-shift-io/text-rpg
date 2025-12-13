use tracing::{error, info};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};
use regex::Regex;

use crate::{services::{command_context::CommandContext, extract::{extract_between, extract_markdown_block}, game_info::GameInfo}};
use crate::globals::*;


const ACT_PROMPT: &str = r#"
I am a dungeon master. I need you to take the current state of the game and update it to reflect a players action.

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
interface GameState {
  rooms: Room[];
  room_connections: RoomConnection[];
  player_characters: PlayerCharacter[];
  objectives: Objective[];
  theme: string;
  theme_description: string;
}

interface Room {
  room_number: number;
  name: string;
  description: string;
  monsters: Monster[];
  items: string[];
  is_start_room: boolean;
  is_end_room: boolean;
}

interface RoomConnection {
  connected_room_numbers: number[];
  connection_type: string;
  description: string;
}

interface Monster {
  name: string;
  description: string;
  abilities: string[];
  items: string[];
  health: number;
  strength: number;
  dexterity: number;
  constitution: number;
  intelligence: number;
  wisdom: number;
}

interface PlayerCharacter {
  name: string;
  character_class: string;
  abilities: string[];
  items: string[];
  room_number: number;
  health: number;
  strength: number;
  dexterity: number;
  constitution: number;
  intelligence: number;
  wisdom: number;
}

interface Objective {
  goal: string;
  items: string[];
  monsters: string[];
  completed: boolean;
}
```

In the rules XML tag below I have included specific rules that must not be violated when changing the game state regardless of what the players action says:
<rules>
The user may not invent items that are not in the game state.
If the user attacks a monster, the monster may retaliate and this should be reflected in the game state JSON.
You must return a valid JSON object satisfying the GameState interface.
Do not add additional fields, you may only modify existing fields.
Do not prompt for further instructions, if you are unsure make your best guess.
</rules>

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
pub async fn act(context: CommandContext) -> Result<(), ()> {
    let sender_player_member = context.sender_room_member().await?;
    let sender_display_name = sender_player_member.display_name().unwrap().to_owned();
    let sender_player_info = context.find_room_member_player_character_info(sender_player_member).await.unwrap();
    if !sender_player_info.is_alive() {
        context.room_send(&format!("🪦 {}", &sender_display_name)).await.unwrap();
        return Ok(());
    }

    // --- Round Logic Start ---
    let mut round_reset = false;
    let mut current_round_number = 0;
    
    {
        let mut round_info_guard = GLOBAL_ROUND_INFO.lock().await;
        if let Some(round_info) = round_info_guard.as_mut() {
            current_round_number = round_info.round_number;
            let has_acted = round_info.acted_players.contains(&sender_display_name);
            let elapsed = round_info.round_start_time.elapsed().unwrap_or(std::time::Duration::ZERO);
            // 6 hours timeout
            let timeout_duration = std::time::Duration::from_secs(6 * 60 * 60);
            let timeout = elapsed > timeout_duration;

            if has_acted {
                if timeout {
                    // Timeout passed: Start new round logic checks
                    round_info.round_number += 1;
                    round_info.acted_players.clear();
                    round_info.round_start_time = std::time::SystemTime::now();
                    round_reset = true;
                    // Proceed to accept action (we will add user to acted_players below)
                    current_round_number = round_info.round_number;
                } else {
                    let remaining = timeout_duration - elapsed;
                    let remaining_secs = remaining.as_secs();
                    let hours = remaining_secs / 3600;
                    let minutes = (remaining_secs % 3600) / 60;
                    
                    let game_info_guard = GLOBAL_GAME_INFO.lock().await;
                    let mut waiting_for = Vec::new();
                    if let Some(game_info) = game_info_guard.as_ref() {
                        for player in &game_info.player_characters {
                            if player.is_alive() && !round_info.acted_players.contains(&player.name) {
                                waiting_for.push(player.name.clone());
                            }
                        }
                    }

                    // Force drop guard to avoid deadlock if we were to hold it while sending (though room_send is async and we are inside a sync block here effectively? No, we are in an async fn, holding a MutexGuard across await point is bad, but room_send is awaited outside or inside?
                    // Wait, logic check: room_send IS async. Accessing GLOBAL_GAME_INFO locks another mutex.
                    // We are holding round_info_guard (Global Round Info Mutex) while trying to lock Global Game Info Mutex.
                    // This is potential deadlock if elsewhere we lock GameInfo then RoundInfo.
                    // Checking other usages... act() usually locks RoundInfo briefly. GameInfo is locked in with_game_info.
                    // Ideally, we should release GameInfo lock before sending.
                    
                    drop(game_info_guard); // Explicit drop just to be safe/clear, though we only needed it for the list.

                    let waiting_list = if waiting_for.is_empty() {
                        "everyone".to_string() 
                    } else {
                        waiting_for.join(", ")
                    };

                    context.room_send(&format!(
                        "You have already acted in round {}. Please wait for: {}. Timeout in {}h {}m.", 
                        round_info.round_number, waiting_list, hours, minutes
                    )).await.unwrap();
                    return Ok(());
                }
            }

            // Optimistically mark as acted
            round_info.acted_players.push(sender_display_name.clone());
        }
    }

    if round_reset {
        context.room_send(&format!("Round timeout! Round {} started.", current_round_number)).await.unwrap();
    }
    // --- Round Logic End ---

    let act_prompt = ACT_PROMPT
        .replace("${game_state}", &context.game_info_as_json().await?)
        .replace("${action}", &context.clean_text())
        .replace("${name}", &sender_display_name);

    let act_response = match context.execute_prompt(act_prompt).await {
        Ok(res) => res,
        Err(_) => {
             // Revert turn on failure
             let mut round_info_guard = GLOBAL_ROUND_INFO.lock().await;
             if let Some(round_info) = round_info_guard.as_mut() {
                 if let Some(pos) = round_info.acted_players.iter().position(|x| *x == sender_display_name) {
                     round_info.acted_players.remove(pos);
                 }
             }
             return Err(());
        }
    };

    let json_strs = extract_markdown_block("json", &act_response)?;
    if json_strs.len() == 0 {
        context.room_send("[act] Failed to get JSON from response.").await.unwrap();
        // Revert turn
        let mut round_info_guard = GLOBAL_ROUND_INFO.lock().await;
        if let Some(round_info) = round_info_guard.as_mut() {
             if let Some(pos) = round_info.acted_players.iter().position(|x| *x == sender_display_name) {
                 round_info.acted_players.remove(pos);
             }
        }
        return Ok(());
    }

    let story_strs = extract_between("<story>", "</story>", &act_response)?;
    if story_strs.len() == 0 {
        context.room_send("[act] Failed to get story block from response.").await.unwrap();
        // Revert turn
        let mut round_info_guard = GLOBAL_ROUND_INFO.lock().await;
        if let Some(round_info) = round_info_guard.as_mut() {
             if let Some(pos) = round_info.acted_players.iter().position(|x| *x == sender_display_name) {
                 round_info.acted_players.remove(pos);
             }
        }
        return Ok(());
    }

    let new_game_info = match GameInfo::from_str(&json_strs[0]) {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to parse JSON: {}", &json_strs[0]);
            context.room_send("Failed to parse JSON.").await.unwrap();
            // Revert turn
            let mut round_info_guard = GLOBAL_ROUND_INFO.lock().await;
            if let Some(round_info) = round_info_guard.as_mut() {
                 if let Some(pos) = round_info.acted_players.iter().position(|x| *x == sender_display_name) {
                     round_info.acted_players.remove(pos);
                 }
            }
            return Ok(());
        }
    };
    
    // Check for round completion based on NEW game info (in case someone died)
    let total_alive_players = new_game_info.player_characters.iter().filter(|p| p.is_alive()).count();
    
    *GLOBAL_GAME_INFO.lock().await = Some(new_game_info.clone());

    context.room_send(&story_strs[0]).await.unwrap();

    // --- Check End of Round ---
    {
        let mut round_info_guard = GLOBAL_ROUND_INFO.lock().await;
        if let Some(round_info) = round_info_guard.as_mut() {
             if round_info.acted_players.len() >= total_alive_players {
                  round_info.round_number += 1;
                  round_info.acted_players.clear();
                  round_info.round_start_time = std::time::SystemTime::now();
                  context.room_send(&format!("All players acted! Round {} started.", round_info.round_number)).await.unwrap();
             }
        }
    }

    Ok(())
}