/* 
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

use crate::{get_ai_chat, services::{command_context::CommandContext, extract::extract_json, game_info::GameInfo}};
use crate::globals::*;

const GAME_UPDATE_RAW_PROMPT: &str = r#"
I am a dungeon master. 

The player with "name" is called "${name}" and has asked me to make the following action:
${action}

The previous state of the game was:
```json
${previous_game_state}
```

The new state of the game after the players action is:
```json
${new_game_state}
```

Now that the player has acted, each monster in the same room as the player has a chance to make an action.
I need to take the current state of the game and update it to reflect the results of all the monsters actions.

Please respond with the updated state of the game.
The format must be the same as the current game state.
I need the response in JSON format.
Do not include comments within the JSON structure.
Do not prompt for further instructions, if you are unsure make your best guess.
"#;

const ACT_STORY_RAW_PROMPT: &str = r#"
I am a dungeon master.

The player with "name" is called "${name}" and has asked me to make the following action:
${action}

I have modified the game state in accordance with the players action. I have then given the monsters in the same room as the player a chance to make an actions.

The previous state of the game was:
```json
${previous_game_state}
```

The new state of the game after the monsters actions is:
```json
${new_game_state}
```

I need you to describe for me the monsters actions as a story.

Please include in your response exact values for things such as damage.
"#;

/// This is a private command. You can't call it directly. It is called from the act commend
pub async fn monster_act(context: CommandContext, previous_game_state: GameInfo, new_game_state: GameInfo, action_prompt: String, acting_player_member: &RoomMember) -> Result<(), ()> {
    context.notify_typing().await;

    let previous_game_state_str = serde_json::to_string_pretty(&previous_game_state).unwrap();
    let new_game_state_str = serde_json::to_string_pretty(&new_game_state).unwrap();

    let game_update_prompt = GAME_UPDATE_RAW_PROMPT
        .replace("${previous_game_state}", &previous_game_state_str)
        .replace("${new_game_state}", &new_game_state_str)
        .replace("${action}", &action_prompt)
        .replace("${name}", acting_player_member.display_name().unwrap());

    let game_info = context.execute_json_prompt::<GameInfo>(game_update_prompt).await?;
    let new_game_state_str = serde_json::to_string_pretty(&game_info).unwrap();

    // update the game state
    let game_info_clone = context.clone_game_info().await?;

    // form a prompt to describe the monsters action as a story
    let act_story_prompt = ACT_STORY_RAW_PROMPT
        .replace("${previous_game_state}", &new_game_state_str)
        .replace("${new_game_state}", &new_game_state_str)
        .replace("${action}", &action_prompt)
        .replace("${name}", acting_player_member.display_name().unwrap());

    context.execute_story_prompt(act_story_prompt).await?;
    Ok(())
}*/