use rltk::{GameState, Point, Rltk};
use specs::*;
use std::{time::Duration, thread::sleep};

use crate::components::{InBackpack, WantsToDropItem, WantsToRemoveItem, WantsToUseItem};
use crate::gamelog;
use crate::map_builders::{build_random_map, map::Map};
use crate::spawner::{
    components::{CombatStats, Equipped, Position, Ranged, Viewshed},
    player::{player_input, Player},
    spawn, SpawnSeed,
};
use crate::systems::{
    damage_system::DamageSystem,
    hunger_system::HungerSystem,
    inventory_systems::{ItemCollectionSystem, ItemDropSystem, ItemRemoveSystem, ItemUseSystem},
    map_indexing_system::MapIndexingSystem,
    melee_combat_system::*,
    monster_ai_system::MonsterAI,
    particle_system::*,
    saveload_system, ui_system,
    trigger_system::*,
    ui_system::*, ui_system::main_menu::*,
    visiblity_system::VisibilitySystem,
};

#[derive(PartialEq, Copy, Clone)]
pub enum RunState {
    AwaitingInput,
    PreRun,
    PlayerTurn,
    MonsterTurn,
    ShowInventory,
    ShowDropItem,
    ShowTargeting {
        range: i32,
        item: Entity,
    },
    MainMenu {
        menu_selection: ui_system::MainMenuSelection,
    },
    SaveGame,
    NextLevel,
    ShowRemoveItem,
    GameOver
}

pub struct State {
    pub ecs: World
}

impl State {
    fn run_systems(&mut self, ctx: &mut Rltk) {
        ctx.cls();        
        cull_dead_particles(&mut self.ecs, ctx);
        let mut uis = UISystem { ctx };
        uis.run_now(&self.ecs);
        let mut vis = VisibilitySystem {};
        vis.run_now(&self.ecs);
        let mut mai = MonsterAI {};
        mai.run_now(&self.ecs);
        let mut tgs = TriggerSystem{};
        tgs.run_now(&self.ecs);
        let mut mis = MapIndexingSystem {};
        mis.run_now(&self.ecs);
        let mut mcs = MeleeCombatSystem {};
        mcs.run_now(&self.ecs);
        let mut dam = DamageSystem {};
        dam.run_now(&self.ecs);
        let mut ins = ItemCollectionSystem {};
        ins.run_now(&self.ecs);
        let mut ius = ItemUseSystem {};
        ius.run_now(&self.ecs);
        let mut ids = ItemDropSystem {};
        ids.run_now(&self.ecs);
        let mut irs = ItemRemoveSystem {};
        irs.run_now(&self.ecs);
        let mut hus = HungerSystem {};
        hus.run_now(&self.ecs);
        let mut pss = ParticleSpawnSystem {};
        pss.run_now(&self.ecs);
        
        self.ecs.maintain();
    }

    fn entities_to_remove_on_level_change(&mut self) -> Vec<Entity> {
        let entities = self.ecs.entities();
        let player = self.ecs.read_storage::<Player>();
        let backpack = self.ecs.read_storage::<InBackpack>();
        let equipped = self.ecs.read_storage::<Equipped>();
        let player_entity = self.ecs.fetch::<Entity>();

        let mut to_delete: Vec<Entity> = Vec::new();
        for entity in entities.join() {
            let mut should_delete = true;

            // Don't delete the player
            let p = player.get(entity);
            if let Some(_p) = p {
                should_delete = false;
            }

            // Don't delete the player's equipment
            let bp = backpack.get(entity);
            if let Some(bp) = bp {
                if bp.owner == *player_entity {
                    should_delete = false;
                }
            }

            // Don't delete wielded equipment.
            let eq = equipped.get(entity);
            if let Some(eq) = eq {
                if eq.owner == *player_entity {
                    should_delete = false;
                }
            }

            if should_delete {
                to_delete.push(entity);
            }
        }

        to_delete
    }

    fn goto_next_level(&mut self) {
        // Delete entities that aren't the player or his/her equipment
        let to_delete = self.entities_to_remove_on_level_change();
        for target in to_delete {
            self.ecs
                .delete_entity(target)
                .expect("Unable to delete entity");
        }
        // Build a new map and place the player
        let worldmap;
        {
            let mut worldmap_resource = self.ecs.write_resource::<Map>();
            let current_depth = worldmap_resource.depth;
            *worldmap_resource = build_random_map(current_depth + 1);
            worldmap = worldmap_resource.clone();
        }
        // Spawn bad guys
        spawn(&mut self.ecs, &worldmap, SpawnSeed::Monster);
        spawn(&mut self.ecs, &worldmap, SpawnSeed::Item);

        // Place the player and update resources
        let (player_x, player_y) = worldmap.get_upstairs();
        let mut player_position = self.ecs.write_resource::<Point>();
        *player_position = Point::new(player_x, player_y);
        let mut position_components = self.ecs.write_storage::<Position>();
        let player_entity = self.ecs.fetch::<Entity>();
        let player_pos_comp = position_components.get_mut(*player_entity);
        if let Some(player_pos_comp) = player_pos_comp {
            player_pos_comp.set_x(player_x);
            player_pos_comp.set_y(player_y);
        }
        // Mark the player's visibility as dirty
        let mut viewshed_components = self.ecs.write_storage::<Viewshed>();
        let vs = viewshed_components.get_mut(*player_entity);
        if let Some(vs) = vs {
            vs.dirty = true;
        }
        // Notify the player and give them some health
        let mut gamelog = self.ecs.fetch_mut::<gamelog::GameLog>();
        gamelog
            .entries
            .push("You descend to the next level, and take a moment to heal.".to_string());
        let mut player_health_store = self.ecs.write_storage::<CombatStats>();
        let player_health = player_health_store.get_mut(*player_entity);
        if let Some(player_health) = player_health {
            player_health.hp = i32::max(player_health.hp, player_health.hp_max / 2);
        }
    }

    fn game_over_cleanup(&mut self) {
        // Delete everything
        let mut to_delete = Vec::new();
        for e in self.ecs.entities().join() {
            to_delete.push(e);
        }
        for del in to_delete.iter() {
            self.ecs.delete_entity(*del).expect("Deletion failed");
        }
    
        // Build a new map and place the player
        let worldmap = build_random_map(1);
        self.ecs.insert(worldmap.clone());
        spawn(&mut self.ecs, &worldmap, SpawnSeed::Player);

        // Spawn bad guys
        spawn(&mut self.ecs, &worldmap, SpawnSeed::Monster);
        spawn(&mut self.ecs, &worldmap, SpawnSeed::Item);
    }
    
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut Rltk) {
        ctx.cls();
        let mut newrunstate;
        {
            let runstate = self.ecs.fetch::<RunState>();
            newrunstate = *runstate;
        }

        match newrunstate {
            RunState::PreRun => {
                self.run_systems(ctx);
                newrunstate = RunState::AwaitingInput;
            }
            RunState::AwaitingInput => {
                cull_dead_particles(&mut self.ecs, ctx);
                let mut pss = ParticleSpawnSystem {};
                pss.run_now(&self.ecs);
                let mut uis = UISystem { ctx };
                uis.run_now(&self.ecs);
                self.ecs.maintain();
                newrunstate = player_input(self, ctx);
            }
            RunState::PlayerTurn => {
                self.run_systems(ctx);
                newrunstate = RunState::MonsterTurn;
            }
            RunState::MonsterTurn => {
                self.run_systems(ctx);
                newrunstate = RunState::AwaitingInput;
            }
            RunState::ShowInventory => {
                let result = ui_system::show_inventory(self, ctx);
                match result.0 {
                    ui_system::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ui_system::ItemMenuResult::NoResponse => {}
                    ui_system::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let is_ranged = self.ecs.read_storage::<Ranged>();
                        let is_item_ranged = is_ranged.get(item_entity);
                        if let Some(is_item_ranged) = is_item_ranged {
                            newrunstate = RunState::ShowTargeting {
                                range: is_item_ranged.range,
                                item: item_entity,
                            };
                        } else {
                            let entities = self.ecs.entities();
                            let players = self.ecs.read_storage::<Player>();
                            let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                            for (entity, _player) in (&entities, &players).join() {
                                intent
                                    .insert(
                                        entity,
                                        WantsToUseItem {
                                            item: item_entity,
                                            target: None,
                                        },
                                    )
                                    .expect("Unable to insert intent");
                            }
                            newrunstate = RunState::PlayerTurn;
                        }
                    }
                }
            }
            RunState::ShowDropItem => {
                let result = ui_system::drop_item_menu(self, ctx);
                match result.0 {
                    ui_system::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ui_system::ItemMenuResult::NoResponse => {}
                    ui_system::ItemMenuResult::Selected => {
                        let entities = self.ecs.entities();
                        let players = self.ecs.read_storage::<Player>();
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToDropItem>();
                        for (entity, _player) in (&entities, &players).join() {
                            intent
                                .insert(entity, WantsToDropItem { item: item_entity })
                                .expect("Unable to insert intent");
                        }
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::ShowTargeting { range, item } => {
                let mut uis = UISystem { ctx };
                uis.run_now(&self.ecs);
                let result = ui_system::ranged_target(self, ctx, range);
                match result.0 {
                    ui_system::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ui_system::ItemMenuResult::NoResponse => {}
                    ui_system::ItemMenuResult::Selected => {
                        let entities = self.ecs.entities();
                        let players = self.ecs.read_storage::<Player>();
                        let mut intent = self.ecs.write_storage::<WantsToUseItem>();
                        for (entity, _player) in (&entities, &players).join() {
                            intent
                                .insert(
                                    entity,
                                    WantsToUseItem {
                                        item,
                                        target: result.1,
                                    },
                                )
                                .expect("Unable to insert intent");
                        }
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::MainMenu { menu_selection: selection } => {
                let result;
                if selection != ui_system::MainMenuSelection::DebugMapCont {
                    result = ui_system::main_menu::main_menu(self, ctx); 
                } else {
                    result = ui_system::main_menu::MainMenuResult::Selected { selected : ui_system::MainMenuSelection::DebugMap };
                }
                
                match result {
                    ui_system::main_menu::MainMenuResult::NoSelection { selected } => {
                        newrunstate = RunState::MainMenu {
                            menu_selection: selected,
                        }
                    }
                    ui_system::main_menu::MainMenuResult::Selected { selected } => match selected {
                        ui_system::MainMenuSelection::NewGame => newrunstate = RunState::PreRun,
                        ui_system::MainMenuSelection::LoadGame => {
                            saveload_system::load_game(&mut self.ecs);
                            newrunstate = RunState::AwaitingInput;
                            saveload_system::delete_save();
                        }
                        ui_system::MainMenuSelection::DebugMap => {
                            let mut map = self.ecs.fetch_mut::<Map>();
                            let snapshot = map.get_snapshot();
                            
                            sleep(Duration::from_millis(500));
                            ctx.cls();
                            match snapshot {
                                Some(t) => { 
                                    map.draw_map(t, ctx); 
                                    newrunstate = RunState::MainMenu { menu_selection: MainMenuSelection::DebugMapCont };
                                },
                                None => {
                                    newrunstate = RunState::MainMenu {
                                        menu_selection: MainMenuSelection::LoadGame,
                                    };
                                }
                            }
                        }
                        ui_system::MainMenuSelection::DebugMapCont => {
                            let mut map = self.ecs.fetch_mut::<Map>();
                            let snapshot = map.get_snapshot();
                            
                            sleep(Duration::from_millis(500));
                            ctx.cls();
                            match snapshot {
                                Some(t) => { 
                                    map.draw_map(t, ctx); 
                                    newrunstate = RunState::MainMenu { menu_selection: MainMenuSelection::DebugMapCont };
                                },
                                None => {
                                    sleep(Duration::from_millis(5000));
                                    newrunstate = RunState::MainMenu {
                                        menu_selection: MainMenuSelection::LoadGame,
                                    };
                                }
                            }
                        }
                        ui_system::MainMenuSelection::Quit => {
                            ::std::process::exit(0);
                        }
                    },
                }
            }
            RunState::SaveGame => {
                saveload_system::save_game(&mut self.ecs);
                newrunstate = RunState::MainMenu {
                    menu_selection: MainMenuSelection::LoadGame,
                };
            }
            RunState::NextLevel => {
                self.goto_next_level();
                newrunstate = RunState::PreRun;
            }
            RunState::ShowRemoveItem => {
                let result = ui_system::remove_item_menu(self, ctx);
                match result.0 {
                    ui_system::ItemMenuResult::Cancel => newrunstate = RunState::AwaitingInput,
                    ui_system::ItemMenuResult::NoResponse => {}
                    ui_system::ItemMenuResult::Selected => {
                        let item_entity = result.1.unwrap();
                        let mut intent = self.ecs.write_storage::<WantsToRemoveItem>();
                        intent
                            .insert(
                                *self.ecs.fetch::<Entity>(),
                                WantsToRemoveItem { item: item_entity },
                            )
                            .expect("Unable to insert intent");
                        newrunstate = RunState::PlayerTurn;
                    }
                }
            }
            RunState::GameOver => {
                let result = ui_system::game_over(ctx);
                match result {
                    ui_system::GameOverResult::NoSelection => {}
                    ui_system::GameOverResult::QuitToMenu => {
                        self.game_over_cleanup();
                        newrunstate = RunState::MainMenu{ menu_selection: ui_system::main_menu::MainMenuSelection::NewGame };
                    }
                }
            }
        }

        {
            let mut runwriter = self.ecs.write_resource::<RunState>();
            *runwriter = newrunstate;
        }
        delete_the_dead(&mut self.ecs);
    }
}
