use rltk::RGB;
use specs::prelude::*;

use crate::components::{InBackpack, SufferDamage, WantsToUseItem};
use crate::gamelog::GameLog;
use crate::map_builders::{common::xy_idx, map::Map};
use crate::spawner::components::{
    AreaOfEffect, CombatStats, Confusion, Consumable, Equippable, Equipped, HungerClock,
    HungerState, InflictsDamage, MagicMapper, Name, Position, ProvidesFood, ProvidesHealing,
};
use crate::systems::particle_system::{OnDeathAction, ParticleBuilder};

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, WantsToUseItem>,
        ReadStorage<'a, Name>,
        ReadStorage<'a, Consumable>,
        ReadStorage<'a, ProvidesHealing>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, AreaOfEffect>,
        WriteStorage<'a, Confusion>,
        ReadStorage<'a, Equippable>,
        WriteStorage<'a, Equipped>,
        WriteStorage<'a, InBackpack>,
        WriteExpect<'a, ParticleBuilder>,
        ReadStorage<'a, Position>,
        WriteStorage<'a, HungerClock>,
        ReadStorage<'a, ProvidesFood>,
        ReadStorage<'a, MagicMapper>,
    );

    #[allow(clippy::cognitive_complexity)]
    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            map,
            entities,
            mut wants_use,
            names,
            consumables,
            healing,
            inflict_damage,
            mut combat_stats,
            mut suffer_damage,
            aoe,
            mut confused,
            equippable,
            mut equipped,
            mut backpack,
            mut particle_builder,
            positions,
            mut hunger_clocks,
            provides_food,
            magic_mapper,
        ) = data;

        for (entity, useitem, position) in (&entities, &wants_use, &positions).join() {
            let mut used_item = true;

            // Targeting
            let mut targets: Vec<Entity> = Vec::new();
            match useitem.target {
                None => {
                    targets.push(*player_entity);
                }
                Some(target) => {
                    let area_effect = aoe.get(useitem.item);
                    match area_effect {
                        None => {
                            // Single target in tile
                            let idx = xy_idx(target.x, target.y);
                            for mob in map.tile_content[idx].iter() {
                                targets.push(*mob);
                            }
                        }
                        Some(area_effect) => {
                            // AoE
                            let mut blast_tiles =
                                rltk::field_of_view(target, area_effect.radius, &*map);
                            blast_tiles.retain(|p| {
                                p.x > 0 && p.x < map.width - 1 && p.y > 0 && p.y < map.height - 1
                            });
                            for tile_idx in blast_tiles.iter() {
                                let idx = xy_idx(tile_idx.x, tile_idx.y);
                                for mob in map.tile_content[idx].iter() {
                                    targets.push(*mob);
                                }
                                particle_builder.request(
                                    tile_idx.x,
                                    tile_idx.y,
                                    rltk::RGB::named(rltk::ORANGE),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('░'),
                                    200.0,
                                    OnDeathAction::NoAction,
                                );
                            }
                        }
                    }
                }
            }

            // If it is equippable, then we want to equip it - and unequip whatever else was in that slot
            let item_equippable = equippable.get(useitem.item);
            match item_equippable {
                None => {}
                Some(can_equip) => {
                    let target_slot = can_equip.slot;
                    let target = targets[0];

                    // Remove any items the target has in the item's slot
                    let mut to_unequip: Vec<Entity> = Vec::new();
                    for (item_entity, already_equipped, name) in
                        (&entities, &equipped, &names).join()
                    {
                        if already_equipped.owner == target && already_equipped.slot == target_slot
                        {
                            to_unequip.push(item_entity);
                            if target == *player_entity {
                                gamelog.entries.push(format!("You unequip {}.", name.name));
                            }
                        }
                    }
                    for item in to_unequip.iter() {
                        equipped.remove(*item);
                        backpack
                            .insert(*item, InBackpack { owner: target })
                            .expect("Unable to insert backpack entry");
                    }

                    // Wield the item
                    equipped
                        .insert(
                            useitem.item,
                            Equipped {
                                owner: target,
                                slot: target_slot,
                            },
                        )
                        .expect("Unable to insert equipped component");
                    backpack.remove(useitem.item);
                    if target == *player_entity {
                        gamelog.entries.push(format!(
                            "You equip {}.",
                            names.get(useitem.item).unwrap().name
                        ));
                    }
                }
            }

            // It it is edible, eat it!
            let item_edible = provides_food.get(useitem.item);
            match item_edible {
                None => {}
                Some(_) => {
                    used_item = true;
                    let target = targets[0];
                    let hc = hunger_clocks.get_mut(target);
                    if let Some(hc) = hc {
                        hc.state = HungerState::WellFed;
                        hc.duration = 20;
                        gamelog.entries.push(format!(
                            "You eat the {}.",
                            names.get(useitem.item).unwrap().name
                        ));
                    }
                }
            }

            // If it heals, apply the healing
            let item_heals = healing.get(useitem.item);
            match item_heals {
                None => {}
                Some(healer) => {
                    used_item = false;
                    for target in targets.iter() {
                        let stats = combat_stats.get_mut(*target);
                        if let Some(stats) = stats {
                            stats.heal(i32::min(stats.hp_max, stats.hp + healer.heal_amount));
                            if entity == *player_entity {
                                gamelog.entries.push(format!(
                                    "You use the {}, healing {} hp.",
                                    names.get(useitem.item).unwrap().name,
                                    healer.heal_amount
                                ));
                            }
                            used_item = true;

                            let pos = positions.get(*target);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.get_x(),
                                    pos.get_y(),
                                    rltk::RGB::named(rltk::GREEN),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('♥'),
                                    200.0,
                                    OnDeathAction::NoAction,
                                );
                            }
                        }
                    }
                }
            }

            // If its a magic mapper...
            let is_mapper = magic_mapper.get(useitem.item);
            match is_mapper {
                None => {}
                Some(_) => {
                    used_item = true;
                    gamelog
                        .entries
                        .push("The map is revealed to you!".to_string());
                    for delta_x in -1..=1 {
                        for delta_y in -1..=1 {
                            if !map.is_magic_mapped(
                                position.get_x() + delta_x,
                                position.get_y() + delta_y,
                            ) {
                                particle_builder.request(
                                    position.get_x() + delta_x,
                                    position.get_y() + delta_y,
                                    RGB::named(rltk::MAGENTA),
                                    RGB::named(rltk::ORANGE),
                                    rltk::to_cp437('*'),
                                    100.0,
                                    OnDeathAction::RevealMap,
                                );
                            }
                        }
                    }
                }
            }

            // If it inflicts damage, apply it to the target cell
            let item_damages = inflict_damage.get(useitem.item);
            match item_damages {
                None => {}
                Some(damage) => {
                    used_item = false;
                    for mob in targets.iter() {
                        SufferDamage::new_damage(&mut suffer_damage, *mob, damage.damage);
                        if entity == *player_entity {
                            let mob_name = names.get(*mob).unwrap();
                            let item_name = names.get(useitem.item).unwrap();
                            gamelog.entries.push(format!(
                                "You use {} on {}, inflicting {} hp.",
                                item_name.name, mob_name.name, damage.damage
                            ));

                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.get_x(),
                                    pos.get_y(),
                                    rltk::RGB::named(rltk::RED),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                    OnDeathAction::NoAction,
                                );
                            }
                        }

                        used_item = true;
                    }
                }
            }

            // Can it pass along confusion? Note the use of scopes to escape from the borrow checker!
            let mut add_confusion = Vec::new();
            {
                let causes_confusion = confused.get(useitem.item);
                match causes_confusion {
                    None => {}
                    Some(confusion) => {
                        used_item = false;
                        for mob in targets.iter() {
                            add_confusion.push((*mob, confusion.turns));
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(useitem.item).unwrap();
                                gamelog.entries.push(format!(
                                    "You use {} on {}, confusing them.",
                                    item_name.name, mob_name.name
                                ));

                                let pos = positions.get(*mob);
                                if let Some(pos) = pos {
                                    particle_builder.request(
                                        pos.get_x(),
                                        pos.get_y(),
                                        rltk::RGB::named(rltk::MAGENTA),
                                        rltk::RGB::named(rltk::BLACK),
                                        rltk::to_cp437('?'),
                                        200.0,
                                        OnDeathAction::NoAction,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            for mob in add_confusion.iter() {
                confused
                    .insert(mob.0, Confusion { turns: mob.1 })
                    .expect("Unable to insert status");
            }

            // If its a consumable, we delete it on use
            if used_item {
                let consumable = consumables.get(useitem.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(useitem.item).expect("Delete failed");
                    }
                }
            }
        }

        wants_use.clear();
    }
}
