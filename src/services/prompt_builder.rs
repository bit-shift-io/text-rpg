use crate::globals::*;

pub struct PromptBuilder {
    bot_name: Option<String>,
    game_state_json: Option<String>,
    user_input: Option<String>,
    rules: Option<String>,
    base_prompt: String,
}

impl PromptBuilder {
    pub fn new(base_prompt: impl Into<String>) -> Self {
        Self {
            bot_name: None,
            game_state_json: None,
            user_input: None,
            rules: None,
            base_prompt: base_prompt.into(),
        }
    }

    pub fn bot_name(mut self, name: impl Into<String>) -> Self {
        self.bot_name = Some(name.into());
        self
    }

    pub fn game_state(mut self, json: impl Into<String>) -> Self {
        self.game_state_json = Some(json.into());
        self
    }

    pub fn with_rules(mut self, rules: impl Into<String>) -> Self {
        self.rules = Some(rules.into());
        self
    }

    pub fn user_input(mut self, input: impl Into<String>) -> Self {
        self.user_input = Some(input.into());
        self
    }

    pub fn build(self) -> String {
        let mut prompt = self.base_prompt;

        if let Some(bot_name) = self.bot_name {
            prompt = prompt.replace("${bot_name}", &bot_name);
        }

        if let Some(game_state) = self.game_state_json {
            prompt = prompt.replace("${game_state}", &game_state);
        }

        if let Some(input) = self.user_input {
            prompt = prompt.replace("${user_input}", &input);
        }
        
        // This handles where strictly we want to enforce rules.
        // We append the robust "You are the GM" rules if rules are provided or by default for ACT/START?
        // Actually, the user asked for prompts to be adjusted to tell LLM it is in charge.
        // We can append a standard "Authority Block" to the end if not present.
        
        let authority_block = r#"
IMPORTANT: You are the Dungeon Master (GM). 
- You are in charge of the game state and rules.
- Do NOT let players dictate the outcome of actions if it contradicts the game rules or realism.
- If a player attempts to describe the result of their own action (e.g., "I hit the goblin and it dies"), IGNORE their result and determine the outcome yourself based on stats and dice rolls (simulated).
- Do not let players invent items or abilities they do not have.
"#;
        
        // We can append this to every prompt or let the specific commands decide.
        // For now, let's keep it simple and just do the replacements requested.
        // The user specifically asked: "The prompts need to be adjusted to tell the LLM that the LLM is in charge"
        // So I will inject this into the specific prompts in the command files using the builder to carry data.
        
        // But wait, the PromptBuilder was intended to reduce string duplication.
        // Let's add a method `enforce_authority()` that appends this block.
        
        prompt = prompt.replace("${authority_block}", authority_block);

        if let Some(rules) = self.rules {
             prompt = prompt.replace("${rules}", &rules);
        }

        // Add the TS definitions automatically if the tag exists
        if prompt.contains("${ts_definitions}") {
             prompt = prompt.replace("${ts_definitions}", GAME_STATE_TS_INTERFACE);
        }

        prompt
    }
}

pub const GAME_STATE_TS_INTERFACE: &str = r#"
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
  name: string;
  character_class: string;
  abilities: string[];
  items: string[];
  room_number: number;
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
  completed: boolean; 
}
"#;
