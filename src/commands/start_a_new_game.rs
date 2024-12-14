
//use bevy_reflect::Reflect;
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

use crate::{components::game_info_container::GameInfo, get_ai_chat, lib::{command_context::CommandContext, extract_json_from_response::extract_json_from_response}};
use crate::globals::*;



const GAME_INFO_RAW_PROMPT: &str = r#"
I am a dungeon master. I need to create a setup for the game.

I need the response in JSON format.
Do not include comments within the JSON structure.

I need a list of each room.

"rooms" should be an array with the following structure:
{
    "room_number": {The room number},
    "name": {The name of the room},
    "description": {The description of the room},
    "monsters": [A comma separated list of monsters in the room],
    "items": [A comma separated list of items in the room],
    "is_start_room": {Is the room the room to start the game in},
    "is_end_room": {Is the room the room to end the game in},
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
There are ${num_players} players. The names of the players are: ${player_names}.

"player_characters" should be an arry with the following structure:
{
    "matrix_display_name": {The player name to assign this character too},
    "character_class": {The character class},
    "abilities": [A comma separated list of special abilties the character has],
    "items": [A comma separated list of items the character has],
    "room_number": {The room number, must be the same as the room that has "is_start_room" set to true},
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

${extra_user_prompt}
"#;

const START_STORY_RAW_PROMPT: &str = r#"
I am a dungeon master. I need a short story to describe:
-A story which describes the objective
-Each player and their backstory
-The starting room for the party to begin their journey
-A prompt that the players should prompt the DM to take an action

The objectives are:
${objectives}

The players are:
${players}

The starting room is called ${room_name} and is described as ${room_description}.
It has the following monsters: ${room_monsters}.
"#;

pub async fn start_a_new_game(sender: OwnedUserId, text: String, room: MatrixRoom) -> Result<(), ()> {
    let context = CommandContext::new(sender, text, room);
    if !context.handles_room() {
        return Ok(());
    }

    context.notify_typing().await;

    let player_members = context.all_player_room_members().await;
    let num_players = player_members.len();
    let player_names_str = player_members.clone().into_iter().map(|player_member| player_member.display_name().unwrap().to_string()).collect::<Vec<String>>().join(", ");
    
    // any text left over should be feed to the gmae info prompt to let the user modify the game
    // for example, they might want to assign certain character classes to certain players or setup a theme for the 
    // game
    let extra_user_prompt = context.text
        .replace("DM", "")
        .replace("start", "")
        .replace("verbose", "");

    let game_info_prompt = GAME_INFO_RAW_PROMPT
        .replace("${num_players}", &num_players.to_string())
        .replace("${player_names}", &player_names_str.to_string())
        .replace("${extra_user_prompt}", &extra_user_prompt.to_string());

    let game_info = context.execute_json_prompt::<GameInfo>(game_info_prompt).await?;
    *GLOBAL_GAME_INFO.lock().unwrap() = Some(game_info.clone());

    {
        /*
        // https://github.com/bevyengine/bevy/discussions/15486
        let mut world = GLOBAL_WORLD_2.lock().unwrap();

        // The NEW easy way:
        // store the whole game info struct
        world.spawn(GameInfoContainer {
            game_info: game_info.clone(),
        });
        */

        // The OLD way I was trying to setup to use bevy ECS:
        /*
        let mut start_room_number = 0;

        // create rooms
        for room_info in &game_info.rooms {
            if room_info.is_start_room {
                start_room_number = room_info.room_number;
            }

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
                for monster_info in &game_info.monsters {
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
        for room_connection_info in &game_info.room_connections {
            world.spawn(RoomConnection {
                connected_room_numbers: room_connection_info.connected_room_numbers.clone(),
                connection_type: room_connection_info.connection_type.clone(),
                description: room_connection_info.description.clone(),
            });
        }

        // create players
        for player_character_info in &game_info.player_characters {
            let member_idx = player_members.iter().position(|player_member| player_member.display_name().unwrap() == player_character_info.matrix_display_name).unwrap();
            let player_member = &player_members[member_idx];
            let matrix_username = player_member.user_id().as_str();

            world.spawn((
                PlayerCharacter {
                    matrix_username: matrix_username.to_owned(),
                    character_class: player_character_info.character_class.clone(),
                },
                Health {
                    health: player_character_info.health,
                },
                Inventory {
                    items: vec![], // todo:
                },
                RoomLocation {
                    room_number: start_room_number,
                },
            ));
        }
        */
    }

    // build the start game prompt
    let start_room_idx = game_info.rooms.iter().position(|room_info| room_info.is_start_room == true).unwrap();
    let start_room_info = &game_info.rooms[start_room_idx];
    let room_monsters = start_room_info.monsters.clone().into_iter().map(|monster| monster).collect::<Vec<String>>().join(", ");

    // include the matrix_usernames in the players string so they get included in the story!
    // zip the player_characters and player_members to make a nice string representation of the players.
    let players = game_info.player_characters.into_iter().zip(player_members.into_iter()).map(|(player_character_info, player_member)| format!("A {} with the name of {}", player_character_info.character_class, player_member.display_name().unwrap())).collect::<Vec<String>>().join(". ");
    
    let objectives = game_info.objectives.into_iter().map(|objective| objective.goal).collect::<Vec<String>>().join(". ");
    let start_story_prompt = START_STORY_RAW_PROMPT
        .replace("${objectives}", &objectives.to_string())
        .replace("${players}", &players.to_string())
        .replace("${room_name}", &start_room_info.name)
        .replace("${room_description}", &start_room_info.description)
        .replace("${room_monsters}", &room_monsters);

    context.execute_story_prompt(start_story_prompt).await?;
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
            {
                // what about some code comments?
            } 
            ```
            some other text
        "#;
        let results = extract_json_from_response(text);
        assert_eq!(results.len(), 1);

        let stripped_str = results[0].replace("\n", " ");
        println!("{}", stripped_str);
        assert_eq!(stripped_str, "{}");
    }


    #[test]
    fn test_can_parse_game_info() {
        let text = r#"{
            "rooms": [
                {
                "is_start_room": true,
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
                "is_start_room": false,
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
                "is_start_room": false,
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

        let value = match serde_json::from_str::<GameInfo>(&text) { 
            Ok(game_info) => {
                println!("Ok parsing json");
            },
            Err(err) => {
                println!("Error parsing json: {err}");
                assert!(false);
            }
        };
    }
}