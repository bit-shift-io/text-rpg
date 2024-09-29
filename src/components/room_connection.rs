use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

#[derive(Component, Reflect, Debug)]
pub struct RoomConnection {
    pub connected_room_numbers: Vec<usize>,
    pub connection_type: String,
    pub description: String,
}
