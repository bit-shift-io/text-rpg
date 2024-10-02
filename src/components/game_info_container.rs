use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;
use serde::{Deserialize, Serialize};
use serde_diff::{Apply, Diff, SerdeDiff};

#[derive(Component, Reflect, Debug)]
pub struct GameInfoContainer {
    pub game_info: GameInfo,
}



#[derive(Serialize, Deserialize, Reflect, Debug, Clone, SerdeDiff, PartialEq)]
pub struct GameInfo {
    pub rooms: Vec<RoomInfo>,
    pub room_connections: Vec<RoomConnectionInfo>,
    pub monsters: Vec<MonsterInfo>,
    pub player_characters: Vec<PlayerCharacterInfo>,
    pub objectives: Vec<ObjectiveInfo>,
}

#[derive(Serialize, Deserialize, Reflect, Debug, Clone, SerdeDiff, PartialEq)]
pub struct RoomInfo {
    pub room_number: usize,
    pub name: String,
    pub description: String,
    pub monsters: Vec<String>,
    pub items: Vec<String>,
    pub is_start_room: bool,
    pub is_end_room: bool,
}

#[derive(Serialize, Deserialize, Reflect, Debug, Clone, SerdeDiff, PartialEq)]
pub struct RoomConnectionInfo {
    pub connected_room_numbers: Vec<usize>,
    pub connection_type: String,
    pub description: String,
}

#[derive(Serialize, Deserialize, Reflect, Debug, Clone, SerdeDiff, PartialEq)]
pub struct MonsterInfo {
    pub name: String,
    pub description: String,
    pub abilities: Vec<String>,
    pub items: Vec<String>,
    pub health: u32,
    pub strength: u32,
    pub dexterity: u32,
    pub constitution: u32,
    pub intelligence: u32,
    pub wisdom: u32,
}

#[derive(Serialize, Deserialize, Reflect, Debug, Clone, SerdeDiff, PartialEq)]
pub struct PlayerCharacterInfo {
    pub matrix_display_name: String,
    pub character_class: String,
    pub abilities: Vec<String>,
    pub items: Vec<String>,
    pub health: u32,
    pub strength: u32,
    pub dexterity: u32,
    pub constitution: u32,
    pub intelligence: u32,
    pub wisdom: u32,
}

#[derive(Serialize, Deserialize, Reflect, Debug, Clone, SerdeDiff, PartialEq)]
pub struct ObjectiveInfo {
    pub goal: String,
    pub items: Vec<String>,
    pub monsters: Vec<String>,
}
