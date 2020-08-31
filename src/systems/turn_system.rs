use rltk::RandomNumberGenerator;
use specs::*;

use crate::components::InCombat;
use crate::state_machine::RunState;
use crate::spawner::components::{Actor, Position};
use crate::spawner::player::Player;
use crate::map_builders::map::Map;

pub struct TurnSystem {}

impl<'a> System<'a> for TurnSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = ( ReadExpect<'a, Map>,
                        ReadStorage<'a, Actor>,
                        ReadStorage<'a, Player>,
                        Entities<'a>,
                        WriteStorage<'a, Position>,
                        WriteExpect<'a, RandomNumberGenerator>,
                        WriteStorage<'a, InCombat>,
                        WriteExpect<'a, RunState>,
                    );
    
    fn run(&mut self, data : Self::SystemData) {
        let (map, actors, player, entities, mut positions, mut rng, mut in_combat, mut runstate) = data;

        let data = (&actors, &entities, &positions, &in_combat).join().collect::<Vec<_>>();
        data.sort_by(|&a, &b| b.3.initiative.cmp(&a.3.initiative));
        for set in data {
            let (_actor, entity, mut pos, mut in_combat) = set;

            // If this is the player, switch to AwaitingInput
            let _p : Option<&Player> = player.get(entity);
            if let Some(_p) = _p {
                let runstate = RunState::AwaitingInput;
            }
        }

    }
}