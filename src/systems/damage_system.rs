use specs::prelude::{Join, System, WriteStorage, ReadStorage, WriteExpect, Entities};

use crate::components::SufferDamage;
use crate::spawner::components::{CombatStats, Position};
use crate::map_builders::{common::xy_idx, map::Map};

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = ( WriteStorage<'a, CombatStats>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, Position>,
                        WriteExpect<'a, Map>,
                        Entities<'a> );

    fn run(&mut self, data : Self::SystemData) {
        let (mut stats, mut damage, positions, mut map, entities) = data;

        #[allow(unused_mut)]
        for (entity, mut stats, damage) in (&entities, &mut stats, &damage).join() {
            stats.damage( damage.amount.iter().sum::<i32>() );
            let pos = positions.get(entity);
            if let Some(pos) = pos {
                let idx = xy_idx(pos.get_x(), pos.get_y());
                map.blood_stains.insert(idx);
            }
        }

        damage.clear();
    }
}