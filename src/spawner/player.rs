use rltk::Point;
use specs::*;
use specs_derive::*;
use serde::{Serialize, Deserialize};
use specs::saveload::{MarkedBuilder, SimpleMarker};

use crate::components::{EntityMoved, WantsToMelee, WantsToPickupItem, SerializeMe};
use crate::gamelog::GameLog;
use crate::map_builders::{common::xy_idx, map::Map, map::TileType};
use crate::spawner::{components::*, item::Item, monster::Monster};
use crate::state_machine::{RunState, State};

#[derive(Component, Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Player {}

impl Player {
    #[allow(dead_code)]
    pub fn new(ecs: &mut World, x: i32, y: i32) {
        let player = ecs.create_entity()
            .with(Actor {})
            .with(Player {})
            .with(Position::new(x, y))
            .with(Renderable {
                glyph: rltk::to_cp437('@'),
                fg: rltk::RGB::named(rltk::YELLOW),
                bg: rltk::RGB::named(rltk::BLACK),
                render_order: 1,
            })
            .with(Name {
                name: "Player".into(),
            })
            .with(BlocksTile {} )
            .with(CombatStats {
                hp: 90,
                hp_max: 90,
                attack: 6,
                defense: 2,
                is_dead: false
            })
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .with(HungerClock { state: HungerState::WellFed, duration: 20 })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
        ecs.insert(player);
        ecs.insert( Point {x, y} );
    }
}

pub fn player_input(gs: &mut State, ctx: &mut rltk::Rltk) -> RunState {
    use rltk::VirtualKeyCode;

    let runstate: RunState;
    {
        runstate = *gs.ecs.fetch::<RunState>();
    }
    if runstate == RunState::ShowInventory { return runstate; }
    // Player movement
    match ctx.key {
        None => {
            return RunState::AwaitingInput;
        } // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(&mut gs.ecs, -1, 0)
            }
            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(&mut gs.ecs, 1, 0)
            }
            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::J => {
                try_move_player(&mut gs.ecs, 0, -1)
            }
            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::K => {
                try_move_player(&mut gs.ecs, 0, 1)
            }
            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::Y => try_move_player(&mut gs.ecs, 1, -1),

            VirtualKeyCode::Numpad7 | VirtualKeyCode::U => try_move_player(&mut gs.ecs, -1, -1),

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(&mut gs.ecs, 1, 1),

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(&mut gs.ecs, -1, 1),

            VirtualKeyCode::G => get_item(&mut gs.ecs),

            VirtualKeyCode::I => return RunState::ShowInventory,

            VirtualKeyCode::D => return RunState::ShowDropItem,

            VirtualKeyCode::Escape => return RunState::SaveGame,

            VirtualKeyCode::R => return RunState::ShowRemoveItem,

            // Skip Turn
            VirtualKeyCode::Numpad5 => return skip_turn(&mut gs.ecs),

            VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),

            // Level changes
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }

            _ => {
                return RunState::AwaitingInput;
            }
        },
    }
    return RunState::PlayerTurn;
}

fn skip_turn(ecs: &mut World) -> RunState {

    let player = ecs.read_storage::<Player>();
    let entities = ecs.entities();
    for (entity, _player) in (&entities, &player).join() {
        let viewshed_components = ecs.read_storage::<Viewshed>();
        let monsters = ecs.read_storage::<Monster>();

        let worldmap_resource = ecs.fetch::<Map>();

        let mut can_heal = true;
        let viewshed = viewshed_components.get(entity).unwrap();
        for tile in viewshed.visible_tiles.iter() {
            let idx = xy_idx(tile.x, tile.y);
            for entity_id in worldmap_resource.tile_content[idx].iter() {
                let mob = monsters.get(*entity_id);
                match mob {
                    None => {}
                    Some(_) => { can_heal = false; }
                }
            }
        }
        
        let hunger_clocks = ecs.read_storage::<HungerClock>();
        let hc = hunger_clocks.get(entity);
        if let Some(hc) = hc {
            match hc.state {
                HungerState::Hungry => can_heal = false,
                HungerState::Starving => can_heal = false,
                _ => {}
            }
        }

        if can_heal {
            let mut health_components = ecs.write_storage::<CombatStats>();
            let player_hp = health_components.get_mut(entity).unwrap();
            player_hp.hp = i32::min(player_hp.hp + 1, player_hp.hp_max);
        }
    }

    RunState::PlayerTurn
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog.entries.push("There is no way down from here.".to_string());
        false
    }
}

fn get_item(ecs: &World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();    

    let mut target_item : Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.get_x() == player_pos.x && position.get_y() == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog.entries.push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup.insert(*player_entity, WantsToPickupItem{ collected_by: *player_entity, item }).expect("Unable to insert want to pickup");
        }
    }
}

pub fn try_move_player(ecs: &mut World, delta_x: i32, delta_y: i32) {
    let mut positions = ecs.write_storage::<Position>();
    let mut players = ecs.write_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();
    let entities = ecs.entities();
    let map = ecs.fetch::<Map>();

    for (pos, _player, viewshed, entity) in
        (&mut positions, &mut players, &mut viewsheds, &entities).join()
    {
        
        if pos.try_move(&map, delta_x, delta_y) {

            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.get_x();
            ppos.y = pos.get_y();
            entity_moved.insert(entity, EntityMoved{}).expect("Unable to insert marker");
            viewshed.dirty = true;
        } else {
            let destination_idx = xy_idx(pos.get_x() + delta_x, pos.get_y() + delta_y);

            for potential_target in map.tile_content[destination_idx].iter() {
                let target = combat_stats.get(*potential_target);
                if let Some(_target) = target {
                    wants_to_melee
                        .insert(
                            entity,
                            WantsToMelee {
                                target: *potential_target,
                            },
                        )
                        .expect("Add target failed");
                    return;
                }
            }
        }
    }
}