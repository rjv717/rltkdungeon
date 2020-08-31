use specs::prelude::*;

use crate::components::{SufferDamage, WantsToMelee};
use crate::gamelog::GameLog;
use crate::spawner::components::{
    CombatStats, DefenseBonus, Equipped, HungerClock, HungerState, MeleePowerBonus, Name, Position,
};
use crate::spawner::player::Player;
use crate::state_machine::RunState;
use crate::systems::particle_system::{OnDeathAction, ParticleBuilder};

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, WantsToMelee>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, MeleePowerBonus>,
        ReadStorage<'a, DefenseBonus>,
        ReadStorage<'a, Equipped>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, HungerClock>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_melee,
            names,
            combat_stats,
            mut inflict_damage,
            mut log,
            melee_power_bonus,
            defense_bonus,
            equipped,
            mut particle_builder,
            positions,
            hunger_clock,
        ) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            if !stats.is_dead {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in
                    (&entities, &melee_power_bonus, &equipped).join()
                {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                }

                let hc = hunger_clock.get(entity);
                if let Some(hc) = hc {
                    if hc.state == HungerState::WellFed {
                        offensive_bonus += 1;
                    }
                }

                let target_stats = combat_stats.get(wants_melee.target);
                match target_stats {
                    Some(statistics) => {
                        if statistics.hp > 0 {
                            let target_name = names.get(wants_melee.target).unwrap();
                            let mut defensive_bonus = 0;
                            for (_item_entity, defense_bonus, equipped_by) in
                                (&entities, &defense_bonus, &equipped).join()
                            {
                                if equipped_by.owner == wants_melee.target {
                                    defensive_bonus += defense_bonus.defense;
                                }
                            }

                            let pos = positions.get(wants_melee.target);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.get_x(),
                                    pos.get_y(),
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                    OnDeathAction::NoAction,
                                );
                            }

                            let damage = i32::max(
                                0,
                                (stats.attack + offensive_bonus)
                                    - (statistics.defense + defensive_bonus),
                            );
                            if damage == 0 {
                                log.entries.push(format!(
                                    "{} is unable to hurt {}",
                                    &name.name, &target_name.name
                                ));
                            } else {
                                log.entries.push(format!(
                                    "{} hits {}, for {} hp.",
                                    &name.name, &target_name.name, damage
                                ));
                                SufferDamage::new_damage(
                                    &mut inflict_damage,
                                    wants_melee.target,
                                    damage,
                                );
                            }
                        }
                    }
                    None => (),
                }
            }
        }

        wants_melee.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();
        let entities = ecs.entities();
        for (entity, stats) in (&entities, &combat_stats).join() {
            if stats.is_dead {
                let player = players.get(entity);
                match player {
                    None => {
                        let victim_name = names.get(entity);
                        if let Some(victim_name) = victim_name {
                            log.entries.push(format!("{} is dead", &victim_name.name));
                        }
                        dead.push(entity)
                    }
                    Some(_) => {
                        let mut runstate = ecs.write_resource::<RunState>();
                        *runstate = RunState::GameOver;
                    }
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
