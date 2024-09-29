// https://abadcafe.wordpress.com/2020/12/13/serializing-bevy-ecs-using-reflect-trait/
// This allows to introspect and serialize components tagged with:


use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

#[derive(Component, Reflect, Debug)]
pub struct Room {
    pub room_number: usize,
    pub name: String,
    pub description: String,
}