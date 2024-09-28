use matrix_sdk::{
    media::{MediaFileHandle, MediaFormat, MediaRequest},
    room::MessagesOptions,
    ruma::{
        events::room::message::{MessageType, RoomMessageEventContent},
        OwnedUserId,
    },
    Room as MatrixRoom, RoomMemberships,
};
use tracing::info;
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::get_ai_chat;
use crate::globals::*;
use crate::components::room::Room;


#[derive(Serialize, Deserialize)]
struct MapInfo {
    rooms: Vec<RoomInfo>,
    room_connections: Vec<RoomConnectionInfo>,
}

#[derive(Serialize, Deserialize)]
struct RoomInfo {
    room_number: usize,
    name: String,
    description: String,
    monsters: Vec<String>,
    items: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct RoomConnectionInfo {
    room_number: usize,
    connected_rooms: Vec<usize>,
}

fn extract_json_from_response(text: &str) -> Vec<&str> {
    // todo: this could be improved, it doesnt match the ending ``` string properly
    let re = Regex::new(r"```json([^(````)]+)```").unwrap();

    let mut results = vec![];
    for (_, [path, lineno, line]) in re.captures_iter(text).map(|c| c.extract()) {
        results.push(line);
        //results.push((path, lineno.parse::<u64>()?, line));
    }

    results
}


pub async fn start_a_new_game(sender: OwnedUserId, text: String, room: MatrixRoom) -> Result<(), ()> {
    room.send(RoomMessageEventContent::notice_plain("Starting a new game. Let me think about for a bit.")).await.unwrap();

    let prompt = r#"
        I am a dungeon master. I need to create a setup for the game.

        I need a list of each room. 
            In each room I need a list of monsters. 
            In each room I need a list of items.
        I need a list of room connections.
        I need a list of monsters with any special abilities they might have.

        Each room should be described with the following structure:

            Room number: The room number.
            Name: The name of the room.
            Description: The description of the room.
            Monsters: A comma separated list of monsters in the room.
            Items: A comma separated list of items in the room.

        Each room connection should be described with the following structure:

            Room Connections: A list of connected room numbers.

        Each monstor should be described with the following structure:

            Monstor name: The name of a monster.
            Description: A short description of the monster.
            Abilities: A comma separated list of special abilties the monster has.


        I need the response in JSON format. "rooms" should be an array with the following structure:
        {
            "room_number": {The room number},
            "name": {The name of the room},
            "description": {The description of the room},
            "monsters": [A comma separated list of monsters in the room],
            "items": [A comma separated list of items in the room]
        }

        "monsters" should be an array with the following structure:
        {
            "name": {The name of a monster},
            "description": {A short description of the monster},
            "abilities": [A comma separated list of special abilties the monster has]
        }

        "room_connections" should be an arry with the following structure:
        {
            "room_number": 1,
            "connected_rooms": [A list of connected room numbers]
        }
    "#;

    if let Ok(result) = get_ai_chat().execute(&None, prompt.to_string(), Vec::new()) {
        // Add the prefix ".response:\n" to the result
        // That way we can identify our own responses and ignore them for context
        info!(
            "Response: {} - {}",
            sender.as_str(),
            result//.replace('\n', " ")
        );

        let json_strs = extract_json_from_response(&result);
        if json_strs.len() == 0 {
            room.send(RoomMessageEventContent::notice_plain("Failed to get JSON from response.")).await.unwrap();
            return Ok(());
        }
        
        if let Ok(map_info) = serde_json::from_str::<String>(&json_strs[0]) {   

            let content = RoomMessageEventContent::notice_plain(result);

            room.send(content).await.unwrap();
        

            //let mut world = GLOBAL_WORLD.lock().unwrap();
            //world.spawn(Room {});


            room.send(RoomMessageEventContent::notice_plain("Okay, a new game is ready. Let's begin.")).await.unwrap();
            //return Ok(())
        } else {
            room.send(RoomMessageEventContent::notice_plain("Failed to parse the map info.")).await.unwrap();
        }
    } else {
        room.send(RoomMessageEventContent::notice_plain("Failed to prompt aichat.")).await.unwrap();
    }

    Ok(())
}