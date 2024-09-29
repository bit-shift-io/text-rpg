use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

#[derive(Component, Reflect, Debug)]
pub struct RoomLocation {
    pub room_number: usize,
}
