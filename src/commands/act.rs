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
use regex::Regex;

use crate::{commands::start_a_new_game::extract_json_from_response, components::{game_info_container::GameInfoContainer, health::Health, inventory::Inventory, item::Item, monster::Monster, player_character::PlayerCharacter, room_connection::RoomConnection, room_location::RoomLocation}, get_ai_chat};
use crate::globals::*;
use crate::components::room::Room;

use super::start_a_new_game::GameInfo;

const GAME_UPDATE_RAW_PROMPT: &str = r#"
I am a dungeon master. A player has asked me to make an action.
I need to take the current state of the game and update it to reflect this users action.

The current state of the game is:
${game_state}

The player has asked me to:
${action}

Please respond with the updated state of the game.
The format must be the same as the current game state.
I need the response in JSON format.
Do not prompt for futher instructions, if you are unsure make your best guess.
"#;

pub async fn act(sender: OwnedUserId, text: String, room: MatrixRoom) -> Result<(), ()> {
    room.send(RoomMessageEventContent::notice_plain("Please wait...")).await.unwrap();

    let verbose = text.contains("verbose");

    let action_prompt = text
        .replace("DM", "")
        .replace("act", "")
        .replace("verbose", "");

    let mut json = "".to_owned();
    {
        // get the GameInfoContainer component from the world
        let world_guard = GLOBAL_WORLD.lock().unwrap(); // Error cause by this line.
        let mut world = world_guard;

        // https://doc.qu1x.dev/bevy_trackball/bevy/ecs/system/struct.SystemState.html
        // https://github.com/bevyengine/bevy/issues/2687
        let mut state: SystemState<(
            Commands,
            Query<&GameInfoContainer>,
        )> = SystemState::new(&mut world);

        let (commands, mut game_info_container_query) = state.get_mut(&mut world);
        let mut game_info_container = game_info_container_query.single_mut();
                
        json = serde_json::to_string(&game_info_container.game_info).unwrap();
    }

    info!("GAME_INFO: {}", json);
    if verbose {
        room.send(RoomMessageEventContent::notice_plain(json.clone())).await.unwrap();
    }

    let game_update_prompt = GAME_UPDATE_RAW_PROMPT
        .replace("${game_state}", &json)
        .replace("${action}", &action_prompt);

    if let Ok(result) = get_ai_chat().execute(&None, game_update_prompt.to_string(), Vec::new()) {
        let json_strs = extract_json_from_response(&result);
        if json_strs.len() == 0 {
            room.send(RoomMessageEventContent::notice_plain("Failed to get JSON from response.")).await.unwrap();
            return Ok(());
        }

        if verbose {
            room.send(RoomMessageEventContent::notice_plain(result)).await.unwrap();
        }

        // todo: update the game state

        // todo: form a prompt to describe the users action as a story
    }
    
    Ok(())
}