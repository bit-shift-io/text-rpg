
use tracing::{error, info};
use matrix_sdk::{
    media::{MediaFileHandle, MediaFormat, MediaRequest},
    room::MessagesOptions,
    ruma::{
        events::room::message::{MessageType, RoomMessageEventContent},
        OwnedUserId,
    },
    Room as MatrixRoom, RoomMemberships,
};
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

fn extract_json_from_response(hay: &str) -> Vec<&str> {
    // todo: this could be improved, it doesnt match the ending ``` string properly
    let re = Regex::new(r"```json([\s\S]+)```").unwrap(); // ``json([^(````)]+)```

    let results: Vec<&str> = re.captures_iter(hay).map(|caps| {
        let (_, [json]) = caps.extract();
        json.trim()
    }).collect();

    results
}




pub async fn start_a_new_game(sender: OwnedUserId, text: String, room: MatrixRoom) -> Result<(), ()> {
    room.send(RoomMessageEventContent::notice_plain("Let me go grab the game and set it up...")).await.unwrap();

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
        /* 
        // Add the prefix ".response:\n" to the result
        // That way we can identify our own responses and ignore them for context
        info!(
            "Response: {} - {}",
            sender.as_str(),
            result.replace('\n', " ")
        );*/

        let json_strs = extract_json_from_response(&result);
        if json_strs.len() == 0 {
            room.send(RoomMessageEventContent::notice_plain("Failed to get JSON from response.")).await.unwrap();
            return Ok(());
        }
        
        info!(
            "MapInfo JSON: {}",
            json_strs[0].replace('\n', " ")
        );

        let value = match serde_json::from_str::<MapInfo>(&json_strs[0]) { 
            Ok(map_info) => {
                let content = RoomMessageEventContent::notice_plain(result);

                room.send(content).await.unwrap();
            
                // https://github.com/bevyengine/bevy/discussions/15486
                let mut world = GLOBAL_WORLD.lock().unwrap(); // Error cause by this line.
                world.spawn(Room {});
    
    
                room.send(RoomMessageEventContent::notice_plain("Okay, a new game is ready. Let's begin.")).await.unwrap();
                //return Ok(())
            },
            Err(err) => {
                error!("Error parsing json: {err}");
            }
        };
/* 
            
        } else {
            room.send(RoomMessageEventContent::notice_plain("Failed to parse the map info.")).await.unwrap();
        }*/
    } else {
        room.send(RoomMessageEventContent::notice_plain("Failed to prompt aichat.")).await.unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate test;
    use test::Bencher;


    #[test]
    fn test_extract_json_from_response() {
        let text = r#"
            blah blah 
            ```json
            {} 
            ```
            some other text
        "#;
        let results = extract_json_from_response(text);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0], "{}");
    }


    #[test]
    fn test_can_parse_map_info() {
        let text = r#"{
            "rooms": [
                {
                "room_number": 1,
                "name": "Entrance Hall",
                "description": "A grand hall with a high vaulted ceiling. Dust covers the mosaic tiles on the floor and cobwebs cling to the corners. Two large, rusted iron doors stand at the far end of the hall.",
                "monsters": [
                    "Giant Spider",
                    "Giant Spider"
                ],
                "items": [
                    "Dusty old tapestry",
                    "Silver key"
                ]
                },
                {
                "room_number": 2,
                "name": "The Armory",
                "description": "Weapons of all kinds line the walls, though most are rusted and dull with age. Broken arrows litter the floor, remnants of a past battle.",
                "monsters": [],
                "items": [
                    "Rusty sword",
                    "Broken bow",
                    "Quiver of arrows (5 arrows)"
                ]
                },
                {
                "room_number": 3,
                "name": "The Throne Room",
                "description": "A massive throne of bone sits atop a dais at the far end of the room. A single flickering torch casts eerie shadows on the walls.",
                "monsters": [
                    "Skeleton King"
                ],
                "items": [
                    "Skeleton King's Crown",
                    "Chest of gold (100 gold pieces)"
                ]
                }
            ],
            "monsters": [
                {
                "name": "Giant Spider",
                "description": "A large, hairy spider with eight eyes and venomous fangs.",
                "abilities": [
                    "Web attack (restrains target)",
                    "Poison bite"
                ]
                },
                {
                "name": "Skeleton King",
                "description": "An animated skeleton clad in rusted armor, wielding a bone-chilling sword.",
                "abilities": [
                    "Undead Fortitude (resistance to bludgeoning damage)",
                    "Fear Aura (causes fear in nearby creatures)"
                ]
                }
            ],
            "room_connections": [
                {
                "room_number": 1,
                "connected_rooms": [
                    2,
                    3
                ]
                },
                {
                "room_number": 2,
                "connected_rooms": [
                    1
                ]
                },
                {
                "room_number": 3,
                "connected_rooms": [
                    1
                ]
                }
            ]
        }"#;

        let value = match serde_json::from_str::<MapInfo>(&text) { 
            Ok(map_info) => {
                println!("Ok parsing json");
            },
            Err(err) => {
                println!("Error parsing json: {err}");
                assert!(false);
            }
        };
    }
}