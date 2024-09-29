use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct PlayerCharacter {
    pub matrix_username: String,
    pub character_type: String,
}