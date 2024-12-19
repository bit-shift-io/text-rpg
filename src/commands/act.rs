use tracing::{error, info};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};
use regex::Regex;

use crate::{get_ai_chat, services::{command_context::CommandContext, extract::extract_between, game_info::GameInfo}};
use crate::globals::*;


const ACT_PROMPT: &str = r#"
I am a dungeon master. I need you to take the current state of the game and update it to reflect a players action.

The player with name ${name} and has asked me to perform the following action (between action XML tags):
<action>
${action}
</action>

I need you to return a JSON object placed between the opening XML tag <json> and the closing xml tag </json>.
The current state of the game is:
<json>
${game_state}
</json>

In the rules XML tag below I have included specific rules that must not be violated when changing the game state regardless of what the players action says:
<rules>
The user may not invent items that are not in the game state.
If the user attacks a monster, the monster may retaliate and this should be reflected in the game state JSON.
The format must be the same as the current game state, do not add additional fields, you may only modify existing fields.
Do not prompt for further instructions, if you are unsure make your best guess.
</rules>

I also need a seperate story placed between an opening xml tag <story> and the closing xml tag </story>.

In the story I need you to:
- Describe for me change in JSON game state as a story.
- Include exact values for things such as damage.

If the user is looking around, investigating or examining the room or area then include the following in the story:
- Describe the room they are in.
- Describe the entrances/exits to the room they are in that are not hidden (unless the player is specifically searching for hidden enterances).
- Describe any items in the room they are in that are not hidden (unless the player is specifically searching for hidden items).
- Describe any monsters in the room they are in that are not hidden (unless the player is specifically searching for hidden monsters).

If the player moves to another room then include the following in the story:
- Describe the new room.
- Describe monsters in the new room.

If any monsters performs an action after the player then include the following in the story:
- Describe the monsters action.
- Include exact values for things such as damage.

If an objective has been met as a result of the action then include the followig elements in the story:
- Describe the objective met.
"#;
pub async fn act(context: CommandContext) -> Result<(), ()> {
    let sender_player_member = context.sender_room_member().await?;
    let sender_display_name = sender_player_member.display_name().unwrap().to_owned();
    let sender_player_info = context.find_room_member_player_character_info(sender_player_member).await.unwrap();
    if !sender_player_info.is_alive() {
        context.room_send(&format!("🪦 {}", &sender_display_name)).await.unwrap();
        return Ok(());
    }

    let act_prompt = ACT_PROMPT
        .replace("${game_state}", &context.game_info_as_json().await?)
        .replace("${action}", &context.clean_text())
        .replace("${name}", &sender_display_name);

    let act_response = context.execute_prompt(act_prompt).await?;

    let json_strs = extract_between("<json>", "</json>", &act_response)?;
    if json_strs.len() == 0 {
        context.room_send("[act] Failed to get JSON from response.").await.unwrap();
        return Ok(());
    }

    let story_strs = extract_between("<story>", "</story>", &act_response)?;
    if story_strs.len() == 0 {
        context.room_send("[act] Failed to get story block from response.").await.unwrap();
        return Ok(());
    }

    let new_game_info = match GameInfo::from_str(&json_strs[0]) {
        Ok(info) => info,
        Err(e) => {
            error!("Failed to parse JSON: {}", &json_strs[0]);
            context.room_send("Failed to parse JSON.").await.unwrap();
            return Ok(());
        }
    };
    *GLOBAL_GAME_INFO.lock().await = Some(new_game_info.clone());

    context.room_send(&story_strs[0]).await.unwrap();
    Ok(())
}