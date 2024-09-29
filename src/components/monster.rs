use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

#[derive(Component, Reflect, Debug)]
pub struct Monster {
    pub name: String,
    pub description: String,
}
