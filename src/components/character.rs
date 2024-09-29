use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Character {
    pub name: String,
    pub description: String,
}