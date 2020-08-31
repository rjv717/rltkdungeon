use specs::prelude::*;

use crate::gamelog::GameLog;
use crate::map_builders::{common::xy_idx, map::Map};
use crate::spawner::components::{EntryTrigger, Hidden, InflictsDamage, Name, Position, SingleActivation};
use crate::systems::particle_system::{OnDeathAction, ParticleBuilder};
use crate::components::{EntityMoved, SufferDamage};

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Map>,
                        WriteStorage<'a, EntityMoved>,
                        ReadStorage<'a, Position>,
                        ReadStorage<'a, EntryTrigger>,
                        ReadStorage<'a, InflictsDamage>,
                        WriteStorage<'a, Hidden>,
                        WriteExpect<'a, ParticleBuilder>,
                        ReadStorage<'a, Name>,
                        WriteStorage<'a, SufferDamage>,
                        ReadStorage<'a, SingleActivation>,
                        Entities<'a>,
                        WriteExpect<'a, GameLog>);

    fn run(&mut self, data : Self::SystemData) {
        let (map, mut entity_moved, position, entry_trigger, inflicts_damage, mut hidden, mut particle_builder, names, mut inflict_damage, single_activation, entities, mut log) = data;

        // Iterate the entities that moved and their final position
        let mut remove_entities : Vec<Entity> = Vec::new();
        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = xy_idx(pos.get_x(), pos.get_y());
            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id { // Do not bother to check yourself for being a trap!
                    let maybe_trigger = entry_trigger.get(*entity_id);
                    match maybe_trigger {
                        None => {},
                        Some(_trigger) => {
                            // We triggered it                            
                            let name = names.get(*entity_id);
                            if let Some(name) = name {
                                log.entries.push(format!("{} triggers!", &name.name));
                            }
                            // If the trap is damage inflicting, do it
                            let damage = inflicts_damage.get(*entity_id);
                            if let Some(damage) = damage {
                                particle_builder.request(pos.get_x(), pos.get_y(), rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('‼'), 200.0, OnDeathAction::NoAction);
                                SufferDamage::new_damage(&mut inflict_damage, entity, damage.damage);
                            }
                            hidden.remove(*entity_id); // The trap is no longer hidden
                            // If it is single activation, it needs to be removed
                            let sa = single_activation.get(*entity_id);
                            if let Some(_sa) = sa {
                                remove_entities.push(*entity_id);
                            }
                        }
                    }
                }
            }
        }

        // Remove any single activation traps
        for trap in remove_entities.iter() {
            entities.delete(*trap).expect("Unable to delete trap");
        }

        // Remove all entity movement markers
        entity_moved.clear();
    }
}