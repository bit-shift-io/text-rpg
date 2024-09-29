
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

use crate::{components::{health::Health, inventory::Inventory, item::Item, monster::Monster, player_character::PlayerCharacter, room_connection::RoomConnection, room_location::RoomLocation}, get_ai_chat};
use crate::globals::*;
use crate::components::room::Room;


#[derive(Serialize, Deserialize)]
struct MapInfo {
    rooms: Vec<RoomInfo>,
    room_connections: Vec<RoomConnectionInfo>,
    monsters: Vec<MonsterInfo>,
    player_characters: Vec<PlayerCharacterInfo>,
    objectives: Vec<ObjectiveInfo>,
}

#[derive(Serialize, Deserialize)]
struct RoomInfo {
    room_number: usize,
    name: String,
    description: String,
    monsters: Vec<String>,
    items: Vec<String>,
    is_start_room: bool,
    is_end_room: bool,
}

#[derive(Serialize, Deserialize)]
struct RoomConnectionInfo {
    connected_room_numbers: Vec<usize>,
    connection_type: String,
    description: String,
}

#[derive(Serialize, Deserialize)]
struct MonsterInfo {
    name: String,
    description: String,
    abilities: Vec<String>,
    items: Vec<String>,
    health: u32,
    strength: u32,
    dexterity: u32,
    constitution: u32,
    intelligence: u32,
    wisdom: u32,
}

#[derive(Serialize, Deserialize)]
struct PlayerCharacterInfo {
    character_type: String,
    abilities: Vec<String>,
    items: Vec<String>,
    health: u32,
    strength: u32,
    dexterity: u32,
    constitution: u32,
    intelligence: u32,
    wisdom: u32,
}

#[derive(Serialize, Deserialize)]
struct ObjectiveInfo {
    goal: String,
    items: Vec<String>,
    monsters: Vec<String>,
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



const RAW_PROMPT: &str = r#"
I am a dungeon master. I need to create a setup for the game.

I need the response in JSON format. 

I need a list of each room.

"rooms" should be an array with the following structure:
{
    "room_number": {The room number},
    "name": {The name of the room},
    "description": {The description of the room},
    "monsters": [A comma separated list of monsters in the room],
    "items": [A comma separated list of items in the room],
    "is_start_room": {Is the room the room to start the game in?},
    "is_end_room": {Is the room the room to end the game in?},
}

I need a list of room connections. This describes which rooms connect to other rooms.
Ensure each room is connected to at least 1 other room.

"room_connections" should be an arry with the following structure:
{
    "connected_room_numbers": [A list of connected room numbers],
    "connection_type": {The type of the connection, e.g. "door" or "portal"},
    "description": {A short description of the connection}
}

I need a list of monsters with any special abilities they might have.

"monsters" should be an array with the following structure:
{
    "name": {The name of a monster},
    "description": {A short description of the monster},
    "abilities": [A comma separated list of special abilties the monster has],
    "health: {The health of the monster},
    "strength": {The start strength of the monster},
    "dexterity": {The start dexterity of the monster},
    "constitution": {The start constitution of the monster},
    "intelligence": {The start intelligence of the monster},
    "wisdom": {The start wisdom of the monster},
    "items": [A comma separated list of items the monster has],
}

I need a list of player characters that describe the character for each player who is playing.
There are ${num_players} players.

"player_characters" should be an arry with the following structure:
{
    "character_type": {The type of character, eg. Barbarian},
    "abilities": [A comma separated list of special abilties the character has],
    "items": [A comma separated list of items the character has],
    "health: {The start health of the character},
    "strength": {The start strength of the character},
    "dexterity": {The start dexterity of the character},
    "constitution": {The start constitution of the character},
    "intelligence": {The start intelligence of the character},
    "wisdom": {The start wisdom of the character},
}

I need a list of objectives for the players to achive together.

"objectives" should be an arry with the following structure:
{
    "goal": {The goal},
    "items": [A comma separated list of items required to complete the objective],
    "monsters: [A comma separated list of monsters required to complete the objective]
}
"#;

pub async fn start_a_new_game(sender: OwnedUserId, text: String, room: MatrixRoom) -> Result<(), ()> {
    room.send(RoomMessageEventContent::notice_plain("Let me go grab the game and set it up...")).await.unwrap();

    let num_players = 2; // todo: fix me - get number of players in the channel
    let prompt = RAW_PROMPT.replace("${num_players}", &num_players.to_string());

    if let Ok(result) = get_ai_chat().execute(&None, prompt.to_string(), Vec::new()) {
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
            
                {
                    // https://github.com/bevyengine/bevy/discussions/15486
                    let mut world = GLOBAL_WORLD.lock().unwrap();

                    // create rooms
                    for room_info in &map_info.rooms {
                        world.spawn(Room {
                            room_number: room_info.room_number,
                            name: room_info.name.clone(),
                            description: room_info.description.clone(),
                        });

                        // create items in the room
                        for item in &room_info.items {
                            world.spawn((
                                Item {
                                    name: item.clone(),
                                },
                                RoomLocation {
                                    room_number: room_info.room_number,
                                },
                            ));
                        }

                        // look up and spawn a monster for each one in this room
                        for monster_name in &room_info.monsters {
                            for monster_info in &map_info.monsters {
                                if monster_info.name == *monster_name {
                                    world.spawn((
                                        Monster {
                                            name: monster_info.name.clone(),
                                            description: monster_info.description.clone()
                                        },
                                        RoomLocation {
                                            room_number: room_info.room_number,
                                        },
                                        Inventory {
                                            items: vec![], // todo:
                                        }
                                    ));
                                }
                            }
                        }
                    }

                    // create room connections
                    for room_connection_info in &map_info.room_connections {
                        world.spawn(RoomConnection {
                            connected_room_numbers: room_connection_info.connected_room_numbers.clone(),
                            connection_type: room_connection_info.connection_type.clone(),
                            description: room_connection_info.description.clone(),
                        });
                    }

                    // create players
                    for player_character_info in &map_info.player_characters {
                        world.spawn((
                            PlayerCharacter {
                                matrix_username: "temp".to_owned(), // todo:
                                character_type: player_character_info.character_type.clone(),
                            },
                            Health {
                                health: player_character_info.health,
                            },
                            Inventory {
                                items: vec![], // todo:
                            }
                        ));
                    }
                }
    
                room.send(RoomMessageEventContent::notice_plain("Okay, a new game is ready. Let's begin.")).await.unwrap();

                // todo: describe the starting room, the quest and each player as a story
            },
            Err(err) => {
                error!("Error parsing json: {err}");
                room.send(RoomMessageEventContent::notice_plain("Failed to parse the map info.")).await.unwrap();
            }
        };
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