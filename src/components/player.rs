use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Player {
    pub matrix_username: String,
    pub character_class: String,
}