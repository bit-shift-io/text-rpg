use tracing::{error, info};
use serde::{de::IntoDeserializer, Deserialize, Serialize};
use regex::Regex;

use crate::{services::{command_context::CommandContext, extract::{extract_between, extract_markdown_block}, game_info::GameInfo, getimgai::get_url_for_prompt}};
use crate::globals::*;

// todo: explore tool use: https://github.com/jeremychone/rust-genai/blob/main/examples/c08-tooluse.rs
// as an alternate to reponding in JSON. Does this do the same thing behinds the scenes anyway?

const START_PROMPT: &str = r#"
I am a dungeon master. I need to create a setup for a new dungeons and dragons role playing game.
The theme of the game is: ${theme_name}.
${theme_description}

I need you to return a JSON object wrapped in a markdown code block with the language "json".
Here is the TypeScript interface for the GameState that I need you to generate:
```typescript
interface GameState {
  theme: string;
  theme_description: string;
  rooms: Room[];
  room_connections: RoomConnection[];
  player_characters: PlayerCharacter[];
  objectives: Objective[];
}

interface Room {
  room_number: number;
  name: string;
  description: string;
  items: string[];
  is_start_room: boolean;
  is_end_room: boolean;
  monsters: Monster[];
}

interface Monster {
  name: string;
  description: string;
  abilities: string[];
  health: number;
  strength: number;
  dexterity: number;
  constitution: number;
  intelligence: number;
  wisdom: number;
  items: string[];
}

interface RoomConnection {
  connected_room_numbers: number[];
  connection_type: string; // e.g. "door" or "portal"
  description: string;
}

interface PlayerCharacter {
  name: string; // Must be one of: ${player_names}
  character_class: string; // Chosen from: ${character_classes}
  abilities: string[];
  items: string[];
  room_number: number; // Must be start room
  health: number;
  strength: number;
  dexterity: number;
  constitution: number;
  intelligence: number;
  wisdom: number;
}

interface Objective {
  goal: string;
  items: string[];
  monsters: string[];
  completed: boolean; // Should start as false
}
```

The `rooms` array should contain the rooms in the dungeon.
The `room_connections` array should describe how rooms are connected.
The `player_characters` array should contain ${num_players} players using the provided names.
The `objectives` array should contain a list of objectives for the players to achieve.

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

    let json_strs = extract_markdown_block("json", &start_response)?;
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
