use bevy_ecs::prelude::*;
use bevy_reflect::Reflect;

use super::item::Item;

#[derive(Component, Reflect, Debug)]
pub struct Inventory {
    pub items: Vec<Item>,
}
