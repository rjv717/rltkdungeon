use rltk::Point;
use specs::prelude::{Entities, Join, ReadExpect, ReadStorage, System, WriteExpect, WriteStorage,};

use crate::components::WantsToMelee;
use crate::spawner::components::{Position, Viewshed, Confusion};
use crate::spawner::{player::Player, monster::Monster};
use crate::map_builders::{map::Map, common::xy_idx};
use crate::state_machine::RunState;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = ( WriteExpect<'a, Map>,
                        ReadExpect<'a, Point>,
                        ReadStorage<'a, Player>,
                        ReadExpect<'a, RunState>,
                        Entities<'a>,
                        WriteStorage<'a, Viewshed>,
                        ReadStorage<'a, Monster>,
                        WriteStorage<'a, Position>,
                        WriteStorage<'a, WantsToMelee>,
                        WriteStorage<'a, Confusion>);

    fn run(&mut self, data : Self::SystemData) {
        let (mut map, player_pos, players, runstate, entities, mut viewshed, monster, mut position, 
            mut wants_to_melee, mut confused) = data;

        if *runstate != RunState::MonsterTurn { return; }

        for (entity, mut viewshed, _monster, pos) in (&entities, &mut viewshed, &monster, &mut position).join() {
            let mut can_act = true;

            let is_confused = confused.get_mut(entity);
            if let Some(i_am_confused) = is_confused {
                i_am_confused.turns -= 1;
                if i_am_confused.turns < 1 {
                    confused.remove(entity);
                }
                can_act = false;
            }

            if can_act {
                let cur_x = pos.get_x();
                let cur_y = pos.get_y();
                let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(cur_x, cur_y), *player_pos);
                if distance < 1.5 {
                    for (player_entity, _player) in (&entities, &players).join() {
                        wants_to_melee.insert(entity, WantsToMelee{ target: player_entity }).expect("Unable to insert attack");
                    }
                }
                if viewshed.visible_tiles.contains(&*player_pos) {
                    // Path to the player
                    let path = rltk::a_star_search(
                        xy_idx(cur_x, cur_y),
                        xy_idx(player_pos.x, player_pos.y),
                        &mut *map
                    );
                    if path.success && path.steps.len() > 1 {
                        let mut idx = xy_idx(cur_x, cur_y);
                        map.blocked[idx] = false;
                        let new_x = path.steps[1] as i32 % map.width;
                        let new_y = path.steps[1] as i32 / map.width;
                        pos.try_move (&map, new_x-cur_x, new_y-cur_y);
                        idx = xy_idx(pos.get_x(), pos.get_y());
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                }
            }
        }
    }
}