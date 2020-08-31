//! Rltk Dungeon
//! 
//! This project began as a project to Learn the Rust Programming Language by following 
//! the Brackets Tutorial to make a simple roguelike. 
use specs::prelude::*;
use specs::World;
use specs::saveload::SimpleMarkerAllocator;
use std::collections::HashMap;

/// Top level components module.
/// 
/// The Components declared here are used to programatically modify Entities at Runtime. 
mod components;
/// The gamelog retains messages to be displayed in the user interface.
/// 
/// Should probably be refactored into the User Interface System.
mod gamelog;
/// map_builders contains all of the construction data and code to create maps.
mod map_builders;
/// spawner organizes and generates all of the Entities that are contained within the Map. 
mod spawner;
/// state_machine manages the state of the Game Loop.
mod state_machine;
/// systems are contained here.
mod systems;

use components::SerializeMe;
use spawner::{spawn, SpawnSeed};
use spawner::components::register_spawns;
use state_machine::{RunState, State};
use systems::{particle_system, ui_system::MainMenuSelection};

/// SHOW_MAPGEN_VISUALIZER is a feature flag to turn on/off the debug map display in the 
/// Main Menu.
const SHOW_MAPGEN_VISUALIZER : bool = true;

fn main() -> rltk::BError {
    use rltk::RltkBuilder;
    let context = RltkBuilder::simple80x50()
        .with_title("Roguelike Tutorial")
        .build()?;
    let mut gs = State {
        ecs: World::new()
    };

    components::register_components(&mut gs.ecs);
    register_spawns(&mut gs.ecs);

    gs.ecs.insert(SimpleMarkerAllocator::<SerializeMe>::new());

    let map = map_builders::build_random_map(1);

    gs.ecs.insert(rltk::RandomNumberGenerator::new());
    gs.ecs.insert(RunState::MainMenu { menu_selection: MainMenuSelection::NewGame} );
    gs.ecs.insert(gamelog::GameLog{ entries : vec!["Welcome to Rusty Roguelike".to_string()] });
    gs.ecs.insert(particle_system::ParticleBuilder::new());
    gs.ecs.insert(particle_system::Particles { particle : HashMap::new() });
    gs.ecs.insert(map.clone());

    spawn(&mut gs.ecs, &map, SpawnSeed::Player);

    spawn(&mut gs.ecs, &map, SpawnSeed::Monster);

    spawn(&mut gs.ecs, &map, SpawnSeed::Item);
 
    rltk::main_loop(context, gs)
}
