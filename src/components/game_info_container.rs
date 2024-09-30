use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

use crate::commands::start_a_new_game::GameInfo;

#[derive(Component, Reflect, Debug)]
pub struct GameInfoContainer {
    pub game_info: GameInfo,
}
