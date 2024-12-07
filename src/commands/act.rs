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

use crate::{commands::monster_act::monster_act, components::{game_info_container::{GameInfo, GameInfoContainer}, health::Health, inventory::Inventory, item::Item, monster::Monster, player_character::PlayerCharacter, room_connection::RoomConnection, room_location::RoomLocation}, get_ai_chat, lib::extract_json_from_response::extract_json_from_response};
use crate::globals::*;
use crate::components::room::Room;

const GAME_UPDATE_RAW_PROMPT: &str = r#"
I am a dungeon master. A player has asked me to make an action.
I need to take the current state of the game and update it to reflect this users action.

The current state of the game is:
```json
${game_state}
```

The player with "matrix_display_name" is called "${matrix_display_name}" and has asked me to:
${action}

Please respond with the updated state of the game.
The format must be the same as the current game state.
I need the response in JSON format.
Do not include comments within the JSON structure.
Do not prompt for further instructions, if you are unsure make your best guess.
"#;

const ACT_STORY_RAW_PROMPT: &str = r#"
I am a dungeon master. A player has asked me to take an action.
I have modified the game state in accordance with the players action but I need you to describe for me the action as a story.

The previous state of the game was:
```json
${previous_game_state}
```

The new state of the game after the players action is:
```json
${new_game_state}
```

The player with "matrix_display_name" is called "${matrix_display_name}" and has asked me to:
${action}

Please include in your response exact values for things such as damage.

If the user is looking around, investigating or examining the room or area, ensure you include descriptions of all the "room_connections" for the room the player is located in.
If the player moves to another room, ensure you describe the new room and any monsters it contains.
"#;

fn is_alive_monster_in_same_room_as_sender(game_info: &GameInfo, acting_player_member: &RoomMember) -> bool {
    let matrix_display_name = acting_player_member.display_name().unwrap();
    let player_character = game_info.player_characters.iter().find(|character| character.matrix_display_name == matrix_display_name).unwrap();
    let room_number = player_character.room_number;
    let room = game_info.rooms.iter().find(|room| room.room_number == room_number).unwrap();
    let monster_in_same_room = room.monsters.len() > 0;
    // todo: check any monsters are alive
    monster_in_same_room
}

pub async fn act(sender: OwnedUserId, text: String, room: MatrixRoom) -> Result<(), ()> {
    room.typing_notice(true).await.unwrap();
    //room.send(RoomMessageEventContent::notice_plain("Please wait...")).await.unwrap();

    let verbose = text.contains("verbose");

    let joined_members = room.members(RoomMemberships::JOIN).await.unwrap();
    let member_idx = joined_members.iter().position(|player_member| player_member.user_id() == sender).unwrap();
    let acting_player_member = &joined_members[member_idx];

    room.typing_notice(true).await.unwrap();

    let action_prompt = text
        .replace("DM", "")
        .replace("act", "")
        .replace("verbose", "");

    let old_game_info = 
    {
        // get the GameInfoContainer component from the world
        let world_guard = GLOBAL_WORLD_2.lock().unwrap(); // Error cause by this line.
        let mut world = world_guard;

        // https://doc.qu1x.dev/bevy_trackball/bevy/ecs/system/struct.SystemState.html
        // https://github.com/bevyengine/bevy/issues/2687
        let mut state: SystemState<(
            Commands,
            Query<&GameInfoContainer>,
        )> = SystemState::new(&mut world);

        let (commands, mut game_info_container_query) = state.get_mut(&mut world);
        let game_info_container = game_info_container_query.single_mut();
         
        game_info_container.game_info.clone()
    };
       
    let json = serde_json::to_string_pretty(&old_game_info).unwrap();

    room.typing_notice(true).await.unwrap();

    let game_update_prompt = GAME_UPDATE_RAW_PROMPT
        .replace("${game_state}", &json)
        .replace("${action}", &action_prompt)
        .replace("${matrix_display_name}", acting_player_member.display_name().unwrap());

    if let Ok(result) = get_ai_chat().execute(&None, game_update_prompt.to_string(), Vec::new()) {
        room.typing_notice(true).await.unwrap();

        let json_strs = extract_json_from_response(&result);
        if json_strs.len() == 0 {
            room.send(RoomMessageEventContent::notice_plain("Failed to get JSON from response.")).await.unwrap();
            return Ok(());
        }

        info!(
            "GAME_INFO JSON: {}",
            json_strs[0].replace('\n', " ")
        );

        let value = match serde_json::from_str::<GameInfo>(&json_strs[0]) { 
            Ok(game_info) => {
                let json_diff = serde_json::to_string(&Diff::serializable(&old_game_info, &game_info)).unwrap();
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

                // form a prompt to describe the users action as a story
                let act_story_prompt = ACT_STORY_RAW_PROMPT
                    .replace("${previous_game_state}", &json)
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

                let alive_monster_in_same_room_as_sender = is_alive_monster_in_same_room_as_sender(&game_info_clone, acting_player_member);
                if alive_monster_in_same_room_as_sender {
                    return monster_act(sender, text, room, old_game_info, game_info_clone, action_prompt, acting_player_member).await;
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