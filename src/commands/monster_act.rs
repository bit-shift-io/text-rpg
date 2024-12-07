use bevy_ecs::{entity::Entity, system::{Commands, Query, SystemState}};
use bevy_reflect::Reflect;
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

use crate::{components::{game_info_container::{GameInfo, GameInfoContainer}, health::Health, inventory::Inventory, item::Item, monster::Monster, player_character::PlayerCharacter, room_connection::RoomConnection, room_location::RoomLocation}, get_ai_chat, lib::extract_json_from_response::extract_json_from_response};
use crate::globals::*;
use crate::components::room::Room;

const GAME_UPDATE_RAW_PROMPT: &str = r#"
I am a dungeon master. 

The player with "matrix_display_name" is called "${matrix_display_name}" and has asked me to make the following action:
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

The player with "matrix_display_name" is called "${matrix_display_name}" and has asked me to make the following action:
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
pub async fn monster_act(sender: OwnedUserId, text: String, room: MatrixRoom, previous_game_state: GameInfo, new_game_state: GameInfo, action_prompt: String, acting_player_member: &RoomMember) -> Result<(), ()> {
    room.typing_notice(true).await.unwrap();

    let verbose = text.contains("verbose");

    let previous_game_state_str = serde_json::to_string_pretty(&previous_game_state).unwrap();
    let new_game_state_str = serde_json::to_string_pretty(&new_game_state).unwrap();

    let game_update_prompt = GAME_UPDATE_RAW_PROMPT
        .replace("${previous_game_state}", &previous_game_state_str)
        .replace("${new_game_state}", &new_game_state_str)
        .replace("${action}", &action_prompt)
        .replace("${matrix_display_name}", acting_player_member.display_name().unwrap());


    if let Ok(result) = get_ai_chat().execute(&None, game_update_prompt.to_string(), Vec::new()) {
        room.typing_notice(true).await.unwrap();

        let json_strs = extract_json_from_response(&result);

        info!(
            "GAME_INFO JSON: {}",
            json_strs[0].replace('\n', " ")
        );

        if verbose {
            room.send(RoomMessageEventContent::notice_plain(json_strs[0])).await.unwrap();
        }

        if json_strs.len() == 0 {
            room.send(RoomMessageEventContent::notice_plain("Failed to get JSON from response.")).await.unwrap();
            return Ok(());
        }

        // todo: this keeps failing as the AI is injecting code comments into the JSON!
        let value = match serde_json::from_str::<GameInfo>(&json_strs[0]) { 
            Ok(game_info) => {
                let json_diff = serde_json::to_string(&Diff::serializable(&new_game_state, &game_info)).unwrap();
                info!("GAME_INFO DIFF: {}", json_diff.replace('\n', " "));

                if verbose {
                    room.send(RoomMessageEventContent::notice_plain(json_diff)).await.unwrap();
                }

                // update the game state
                let game_info_clone = {
                    // get the GameInfoContainer component from the world
                   let world_guard = GLOBAL_WORLD_2.lock().unwrap(); // Error cause by this line.
                   let mut world = world_guard;

                   // https://doc.qu1x.dev/bevy_trackball/bevy/ecs/system/struct.SystemState.html
                   // https://github.com/bevyengine/bevy/issues/2687
                   let mut state: SystemState<(
                       Commands,
                       Query<&mut GameInfoContainer>,
                   )> = SystemState::new(&mut world);

                   let (commands, mut game_info_container_query) = state.get_mut(&mut world);

                   let mut game_info_container = game_info_container_query.single_mut();
                   game_info_container.game_info = game_info;
                   game_info_container.game_info.clone()
                };

                // form a prompt to describe the monsters action as a story
                let act_story_prompt = ACT_STORY_RAW_PROMPT
                    .replace("${previous_game_state}", &new_game_state_str)
                    .replace("${new_game_state}", &json_strs[0])
                    .replace("${action}", &action_prompt)
                    .replace("${matrix_display_name}", acting_player_member.display_name().unwrap());

                room.typing_notice(true).await.unwrap();

                if let Ok(result) = get_ai_chat().execute(&None, act_story_prompt.to_string(), Vec::new()) {
                    info!( "ACT_STORY: {}", result);
                    room.send(RoomMessageEventContent::notice_plain(result)).await.unwrap();
                } else {
                    room.send(RoomMessageEventContent::notice_plain(action_prompt.clone())).await.unwrap();
                }
            },
            Err(err) => {
                error!("Error parsing json: {err}");
                room.send(RoomMessageEventContent::notice_plain("Failed to parse the map info.")).await.unwrap();
            }
        };
    }

    room.typing_notice(false).await.unwrap();
    Ok(())
}