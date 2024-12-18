use tracing::{error, info};
use matrix_sdk::{
    media::{MediaFileHandle, MediaFormat, MediaRequest},
    room::{MessagesOptions, RoomMember},
    ruma::{
        api::client::membership::joined_members, events::room::message::{MessageType, RoomMessageEventContent}, OwnedUserId
    },
    Room as MatrixRoom, RoomMemberships,
};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};
use regex::Regex;

use crate::{get_ai_chat, services::{command_context::CommandContext, extract::{extract_blocks, extract_json, RE_EXTRACT_STORY_BLOCK}, game_info::GameInfo}};
use crate::globals::*;

use super::monster_act::monster_act;

const ACT_PROMPT: &str = r#"
I am a dungeon master.
I need to take the current state of the game and update it to reflect a players action.

The current state of the game is:
```json
${game_state}
```

```story
```

The player with name ${name} and has asked me to:
${action}

In the "story" block I need you to describe for me the action as a story.
Please include in "story" exact values for things such as damage.
If the user is looking around, investigating or examining the room or area, ensure you include descriptions of all the "room_connections" for the room the player is located in.
If the player moves to another room, ensure you describe the new room and any monsters it contains.

Please respond with the updated state of the game.
The format must be the same as the current game state, do not add additional fields, you may only modify existing fields.
I need the response in JSON format.
Do not prompt for further instructions, if you are unsure make your best guess.
"#;

fn is_alive_monster_in_same_room_as_sender(game_info: &GameInfo, acting_player_member: &RoomMember) -> bool {
    let name = acting_player_member.display_name().unwrap();
    let player_character = game_info.player_characters.iter().find(|character| character.name == name).unwrap();
    let room_number = player_character.room_number;
    let room = game_info.rooms.iter().find(|room| room.room_number == room_number).unwrap();
    let monster_in_same_room = room.monsters.len() > 0;
    // todo: check any monsters are alive
    monster_in_same_room
}

pub async fn act(context: CommandContext) -> Result<(), ()> {
    let acting_player_member = context.sender_room_member().await?;
    
    let action_prompt = context.text
        .replace("DM", "")
        .replace("act", "")
        .replace("verbose", "");
  
    let old_game_info = context.clone_game_info().await?;
    let old_game_info_json = serde_json::to_string_pretty(&old_game_info).unwrap();

    let act_prompt = ACT_PROMPT
        .replace("${game_state}", &old_game_info_json)
        .replace("${action}", &action_prompt)
        .replace("${name}", acting_player_member.display_name().unwrap());

    let act_response = context.execute_prompt(act_prompt).await?;

    let json_strs = extract_json(&act_response)?;
    if json_strs.len() == 0 {
        context.room.send(RoomMessageEventContent::notice_plain("[act] Failed to get JSON from response.")).await.unwrap();
        return Ok(());
    }

    let story_strs = extract_blocks(&RE_EXTRACT_STORY_BLOCK, &act_response)?;
    if story_strs.len() == 0 {
        context.room.send(RoomMessageEventContent::notice_plain("[act] Failed to get story block from response.")).await.unwrap();
        return Ok(());
    }

    let new_game_info = GameInfo::from_str(&json_strs[0])?; // todo: proper error handling here
    *GLOBAL_GAME_INFO.lock().await = Some(new_game_info.clone());

    context.room.send(RoomMessageEventContent::notice_plain(story_strs[0].clone())).await.unwrap();

    let alive_monster_in_same_room_as_sender = is_alive_monster_in_same_room_as_sender(&new_game_info, &acting_player_member);
    if alive_monster_in_same_room_as_sender {
        return monster_act(context, old_game_info, new_game_info, action_prompt, &acting_player_member).await;
    }

    Ok(())
}