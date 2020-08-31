use specs::prelude::*;

use crate::spawner::components::{Position, BlocksTile};
use crate::map_builders::{map::Map, common::xy_idx};

pub struct MapIndexingSystem {}

impl<'a> System<'a> for MapIndexingSystem {
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, BlocksTile>,
                        Entities<'a>,);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, positions, blockers, entities) = data;

        map.populate_blocked();
        map.clear_content_index();
        for (entity, position) in (&entities, &positions).join() {
            let idx = xy_idx(position.get_x(), position.get_y());

            // If they block, update the blocking list
            let _p : Option<&BlocksTile> = blockers.get(entity);
            if let Some(_p) = _p {
                map.blocked[idx] = true;
            }

            // Push the entity to the appropriate index slot. It's a Copy
            // type, so we don't need to clone it (we want to avoid moving it out of the ECS!)
            map.tile_content[idx].push(entity);
        }
    }
}