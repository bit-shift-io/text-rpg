use tracing::{error, info};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};
use regex::Regex;

use crate::{get_ai_chat, services::{command_context::CommandContext, extract::extract_between, game_info::GameInfo}};
use crate::globals::*;


const ACT_PROMPT: &str = r#"
I am a dungeon master. I need you to take the current state of the game and update it to reflect a players action.

The player with name ${name} and has asked me to perform the following action:
${action}.

I need you to return a JSON object placed between the opening XML tag <json> and the closing xml tag </json>.
The current state of the game is:
<json>
${game_state}
</json>

If the user attacks a monster, the monster also performs an action and this should be reflected in the game state JSON.

The format must be the same as the current game state, do not add additional fields, you may only modify existing fields.
Do not prompt for further instructions, if you are unsure make your best guess.

I also need a seperate story placed between an opening xml tag <story> and the closing xml tag </story>.

In the story I need you to:
- Describe for me the players action as a story.
- Describe the consequences of the players actions, which should align with the changes you make to the JSON.
- Include exact values for things such as damage.

If the user is looking around, investigating or examining the room or area, then include the following in the story:
- Describe the room they are in.
- Describe the entrances/exits to the room they are in that are not hidden (unless the player is specifically searching for hidden enterances).
- Describe any items in the room they are in that are not hidden (unless the player is specifically searching for hidden items).
- Describe any monsters in the room they are in that are not hidden (unless the player is specifically searching for hidden monsters).

If the player moves to another room, then include the following in the story:
- Describe the new room.
- Describe monsters in the new room.

If any monsters performs an action after the player, then include the following in the story:
- Describe the monsters action.
- Include exact values for things such as damage.
"#;
pub async fn act(context: CommandContext) -> Result<(), ()> {
    let acting_player_member = context.sender_room_member().await?;
    
    let action_prompt = context.text
        .replace(".act", "")
        .replace("verbose", "");
  
    let old_game_info = context.clone_game_info().await?;
    let old_game_info_json = serde_json::to_string_pretty(&old_game_info).unwrap();

    let act_prompt = ACT_PROMPT
        .replace("${game_state}", &old_game_info_json)
        .replace("${action}", &action_prompt)
        .replace("${name}", acting_player_member.display_name().unwrap());

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