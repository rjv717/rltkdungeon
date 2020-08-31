use specs::prelude::*;

use crate::components::{InBackpack, WantsToDropItem};
use crate::gamelog::GameLog;
use crate::spawner::components::{Name, Position};
use crate::spawner::player::Player;

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadStorage<'a, Player>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, WantsToDropItem>,
        ReadStorage<'a, Name>,
        WriteStorage<'a, Position>,
        WriteStorage<'a, InBackpack>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player, mut gamelog, entities, mut wants_drop, names, mut positions, mut backpack) =
            data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos: Position = Position::new(0, 0);
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.set_x(dropped_pos.get_x());
                dropper_pos.set_y(dropped_pos.get_y());
            }
            positions
                .insert(
                    to_drop.item,
                    Position::new(dropper_pos.get_x(), dropper_pos.get_y()),
                )
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);

            // If this is the player, tell us they dropped the item
            let _p: Option<&Player> = player.get(entity);
            if let Some(_p) = _p {
                gamelog.entries.push(format!(
                    "You drop the {}.",
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }

        wants_drop.clear();
    }
}
