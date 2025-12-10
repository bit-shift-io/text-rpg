use tracing::{error, info};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use regex::Regex;

use crate::{services::{command_context::CommandContext, extract::extract_between, game_info::GameInfo, getimgai::get_url_for_prompt}};
use crate::globals::*;

// todo: explore tool use: https://github.com/jeremychone/rust-genai/blob/main/examples/c08-tooluse.rs
// as an alternate to reponding in JSON. Does this do the same thing behinds the scenes anyway?

const START_PROMPT: &str = r#"
I am a dungeon master. I need to create a setup for a new dungeons and dragons role playing game.
The theme of the game is: ${theme_name}.
${theme_description}

I need you to return a JSON object placed between the opening XML tag <json> and the closing xml tag </json>.
Here is the schema I need the JSON wrapped in XML tags:
<json>
{
    "theme": "${theme_name}",
    "theme_description": "${theme_description}",
    "rooms": [
        // An array containing objects with following structure:
        {
            "room_number": {The room number},
            "name": {The name of the room},
            "description": {The description of the room},
            "items": [A comma separated list of items in the room],
            "is_start_room": {Is the room the room to start the game in},
            "is_end_room": {Is the room the room to end the game in},

            "monsters": [
                // An array containing objects with following structure:
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
            ]
        }
    ],

    "room_connections": [
        // An array containing objects with following structure:
        {
            "connected_room_numbers": [A list of connected room numbers],
            "connection_type": {The type of the connection, e.g. "door" or "portal"},
            "description": {A short description of the connection}
        }
    ],

    "player_characters": [
        // An array with ${num_players} players. The names of the players are: ${player_names}.
        // The player objects have the following structure:
        {
            "name": {The player name to assign this character too},
            "character_class": {The character class, chosen from: ${character_classes}},
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
    ],

    "objectives": [
        // I need a list of objectives for the players to achieve together.
        // The objective objects have the following structure:
        {
            "goal": {The goal},
            "items": [A comma separated list of items required to complete the objective],
            "monsters": [A comma separated list of monsters required to complete the objective],
            "completed": false,
        }
    ]
}
</json>

I also need a seperate story placed between an opening xml tag <story> and the closing xml tag </story>.

This succinct story must include:
- A short story which describes the objectives
- Each player and their backstory
- The starting room
- Any monsters in the starting room

The players have provided the following additional information:
${extra_user_prompt}
"#;

pub async fn start(context: CommandContext) -> Result<(), ()> {
    let player_members = context.all_player_room_members().await;
    let num_players = player_members.len();
    let player_names_str = player_members.clone().into_iter().map(|player_member| player_member.display_name().unwrap().to_string()).collect::<Vec<String>>().join(", ");

    let theme = crate::themes::get_random_theme(num_players);
    info!("Starting game with theme: {}", theme.name);

    let start_prompt = START_PROMPT
        .replace("${num_players}", &num_players.to_string())
        .replace("${player_names}", &player_names_str.to_string())
        .replace("${theme_name}", &theme.name)
        .replace("${theme_description}", &theme.description)
        .replace("${character_classes}", &theme.classes.join(", "))
        .replace("${extra_user_prompt}", &context.clean_text());
    let start_response = context.execute_prompt(start_prompt).await?;

    let json_strs = extract_between("<json>", "</json>", &start_response)?;
    if json_strs.len() == 0 {
        context.room_send("[start] Failed to get JSON from response.").await.unwrap();
        return Ok(());
    }

    let story_strs = extract_between("<story>", "</story>", &start_response)?;
    if story_strs.len() == 0 {
        context.room_send("[start] Failed to get story block from response.").await.unwrap();
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

    // Initialize round info
    *GLOBAL_ROUND_INFO.lock().await = Some(RoundInfo {
        round_number: 1,
        acted_players: Vec::new(),
        round_start_time: std::time::SystemTime::now(),
    });
    context.room_send(&format!("Round {} started.", 1)).await.unwrap();

    /*
    // try to generate an image for the story
    let url = get_url_for_prompt(&story_strs[0]).await?;
    context.room_send(&url).await.unwrap();
    */

    Ok(())
}
