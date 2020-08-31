use rltk::RandomNumberGenerator;
use specs::*;

use crate::components::InCombat;
use crate::spawner::components::{Actor, Position};
use crate::map_builders::{ common::xy_idx, map::Map };

pub struct InitiativeSystem {
    dirty: bool,
}

impl InitiativeSystem {
    pub fn new() -> InitiativeSystem {
        InitiativeSystem { dirty: true}
    }
}

impl<'a> System<'a> for InitiativeSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Map>,
                        ReadStorage<'a, Actor>,
                        Entities<'a>,
                        ReadStorage<'a, Position>,
                        WriteExpect<'a, RandomNumberGenerator>,
                        WriteStorage<'a, InCombat>,
                    );
    
    fn run(&mut self, data : Self::SystemData) {
        let (map, actors, entities, positions, mut rng, mut in_combat) = data;

        if self.dirty {
            for (entity, _actor, pos) in (&entities, &actors, &positions).join() {
                let idx = xy_idx(pos.get_x(), pos.get_y());

                if map.visible_tiles[idx] == true {
                    in_combat.insert(entity, InCombat { initiative: rng.roll_dice(1, 20) as i32}).expect("Unable to insert InCombat");
                }
            }
        }
    }
}