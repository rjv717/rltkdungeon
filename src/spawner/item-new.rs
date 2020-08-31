use rltk::{RandomNumberGenerator, *};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use specs_derive::*;

use crate::components::SerializeMe;
use crate::map_builders::map::Map;
use crate::spawner::{components::*, random_table::RandomTable};

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Item {}

#[derive(Default, Clone, Copy)]
pub struct Item_builder {
    
}
pub trait spawner {
    fn create (ecs: &mut rltk::World, x: i32, y: i32) -> Item_Builder;
    fn randomize () -> Item_builder;
}

fn item_table(map: &Map) -> RandomTable {
    RandomTable::new()
        .add("Healing Potion", 7)
        .add("Fireball Scroll", 2)
        .add("Confusion Scroll", 1 + map.depth)
        .add("Magic Missle Scroll", 2)
        .add("Dagger", 1)
        .add("Shield", 1)
        .add("Longsword", map.depth - 1)
        .add("Tower Shield", map.depth - 1)
        .add("Rations", 7)
        .add("Magic Mapping Scroll", map.depth/2 - 1 )
        .add("Bear Trap", 1)
}