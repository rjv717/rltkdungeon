use rltk::RandomNumberGenerator;

/// Provides common functions used in the generation of a Map.
pub mod common;
/// Contains the core structues to store and create a Map.
pub mod map;
/// Rect is a simple helper to create rectangluar spaces within the Map.
pub mod rect;

mod bsp_dungeon_builder;
mod bsp_interior;
mod cellular_automata;
mod drunkards_walk;
mod dla;
mod maze;
//mod prefab_builder;
mod simple_map;
mod voronoi;
mod wave_function_collapse;

use crate::spawner::random_table::*;
use common::Symmetry;
use map::Map;
use bsp_dungeon_builder::BSPDungeonBuilder;
use bsp_interior::BSPInteriorBuilder;
use cellular_automata::CellularAutomataBuilder;
use drunkards_walk::DrunkardsWalkBuilder;
use dla::DLABuilder;
use maze::MazeBuilder;
//use prefab_builder::prefab_levels;
use simple_map::SimpleMapBuilder;
use voronoi::VoronoiBuilder;
use wave_function_collapse::WaveFunctionCollapseBuilder;

#[derive(PartialEq, Copy, Clone)]
pub enum DrunkSpawnMode {
    StartingPoint,
    Random,
}

#[derive(PartialEq, Copy, Clone)]
pub enum DLAAlgorithm { WalkInwards, WalkOutwards, CentralAttractor }

/*
#[derive(PartialEq, Clone)]
#[allow(dead_code)]
pub enum PrefabMode { 
    RexLevel{ template : &'static str },
    Constant{ level : prefab_levels::PrefabLevel },
    None
}*/

#[derive(Clone)]
pub struct BuilderSettings {
    pub spawn_mode: Option<DrunkSpawnMode>,
    pub lifetime: Option<i32>,
    pub floor_percent: Option<f32>,
    pub algorithm: Option<DLAAlgorithm>,
    pub symmetry: Option<Symmetry>,
    pub brush_size: Option<i32>,
//    pub mode : PrefabMode,
}

/// Define a standard interface for creation maps.
trait MapBuilder {
    fn build(&mut self) -> map::Map;
    fn with_settings(&mut self, settings: BuilderSettings) -> Box<dyn MapBuilder>;
    fn add(&mut self, map: Map) -> Box<dyn MapBuilder>;
    fn new(new_depth: i32) -> Box<dyn MapBuilder>
    where
        Self: Sized;
}

pub fn build_random_map(new_depth: i32) -> map::Map {
    let mut rng = RandomNumberGenerator::new();
    let table = map_table();
    let map;
    let mut wave_possible = true;

    match table.roll(&mut rng).as_ref() {
        "Simple Map" => {
            wave_possible = false;
            let mut builder = SimpleMapBuilder::new(new_depth);
            map = builder.build();
        }
        "BSP Dungeon Builder" => {
            let mut builder = BSPDungeonBuilder::new(new_depth);
            map = builder.build();
        }
        "BSP Interior" => {
            let mut builder = BSPInteriorBuilder::new(new_depth);
            map = builder.build();
        }
        "Cellular Automata" => {
            let mut builder = CellularAutomataBuilder::new(new_depth);
            map = builder.build();
        }
        "Drunkards Walk - Open Area" => {
            let mut builder = DrunkardsWalkBuilder::new(new_depth);
            map = builder.build();
        }
        "Drunkards Walk - Open Halls" => {
            let mut builder = DrunkardsWalkBuilder::new(new_depth).with_settings(BuilderSettings {
                spawn_mode: Some(DrunkSpawnMode::Random),
                lifetime: Some(400),
                floor_percent: Some(0.5),
                algorithm: None,
                symmetry: Some(Symmetry::None),
                brush_size: Some(1),
                //mode: PrefabMode::None,
            });
            map = builder.build();
        }
        "Drunkards Walk - Winding Passages" => {
            let mut builder = DrunkardsWalkBuilder::new(new_depth).with_settings(BuilderSettings {
                spawn_mode: Some(DrunkSpawnMode::Random),
                lifetime: Some(100),
                floor_percent: Some(0.4),
                algorithm: None,
                symmetry: Some(Symmetry::None),
                brush_size: Some(1),
                //mode: PrefabMode::None,
            });
            map = builder.build();
        }
        "Drunkards Walk - Fat Passages" => {
            let mut builder = DrunkardsWalkBuilder::new(new_depth).with_settings(BuilderSettings {
                spawn_mode: Some(DrunkSpawnMode::Random),
                lifetime: Some(150),
                floor_percent: Some(0.45),
                algorithm: None,
                symmetry: Some(Symmetry::None),
                brush_size: Some(2),
                //mode: PrefabMode::None,
            });
            map = builder.build();
        }
        "Drunkards Walk - Fearful Symmetry" => {
            let mut builder = DrunkardsWalkBuilder::new(new_depth).with_settings(BuilderSettings {
                spawn_mode: Some(DrunkSpawnMode::Random),
                lifetime: Some(100),
                floor_percent: Some(0.4),
                algorithm: None,
                symmetry: Some(Symmetry::Both),
                brush_size: Some(2),
                //mode: PrefabMode::None,
            });
            map = builder.build();
        }
        "Maze" => {
            wave_possible = false;
            let mut builder = MazeBuilder::new(new_depth);
            map = builder.build();
        }
        "DLA - Walk Inwards" => {
            let mut builder = DLABuilder::new(new_depth).with_settings(BuilderSettings {
                spawn_mode: None,
                lifetime: None,
                floor_percent: Some(0.25),
                algorithm: Some(DLAAlgorithm::WalkInwards),
                symmetry: Some(Symmetry::None),
                brush_size: Some(1),
                //mode: PrefabMode::None,
            });
            map = builder.build();
        }
        "DLA - Walk Outwards" => {
            let mut builder = DLABuilder::new(new_depth).with_settings(BuilderSettings {
                spawn_mode: None,
                lifetime: None,
                floor_percent: Some(0.25),
                algorithm: Some(DLAAlgorithm::WalkOutwards),
                symmetry: Some(Symmetry::None),
                brush_size: Some(2),
                //mode: PrefabMode::None,
            });
            map = builder.build();
        }
        "DLA - Central Attractor" => {
            let mut builder = DLABuilder::new(new_depth).with_settings(BuilderSettings {
                spawn_mode: None,
                lifetime: None,
                floor_percent: Some(0.25),
                algorithm: Some(DLAAlgorithm::CentralAttractor),
                symmetry: Some(Symmetry::None),
                brush_size: Some(2),
                //mode: PrefabMode::None,
            });
            map = builder.build();
        }
        "DLA - Insectoid" => {
            let mut builder = DLABuilder::new(new_depth).with_settings(BuilderSettings {
                spawn_mode: None,
                lifetime: None,
                floor_percent: Some(0.25),
                algorithm: Some(DLAAlgorithm::CentralAttractor),
                symmetry: Some(Symmetry::Horizontal),
                brush_size: Some(2),
                //mode: PrefabMode::None,
            });
            map = builder.build();
        }
        "Voronoi Map Builder" => {
            let mut builder = VoronoiBuilder::new(new_depth);
            map = builder.build();
        }
        _ => {
            let mut builder = SimpleMapBuilder::new(new_depth);
            map = builder.build();
        }
    }

    if rng.roll_dice(1, 3)==1 && wave_possible {
        return WaveFunctionCollapseBuilder::new(new_depth).add(map.clone()).build();
    }

    map
}

fn map_table() -> RandomTable {
    RandomTable::new()
        .add("BSP Dungeon Builder", 4)
        .add("Simple Map", 4)
        .add("BSP Interior", 2)
        .add("Cellular Automata", 2)
        .add("Drunkards Walk - Open Area", 1)
        .add("Drunkards Walk - Open Halls", 4)
        .add("Drunkards Walk - Winding Passages", 4)
        .add("Drunkards Walk - Fat Passages", 4)
        .add("Drunkards Walk - Fearful Symmetry", 4)
        .add("Maze", 2)
        .add("DLA - Walk Inwards", 4)
        .add("DLA - Walk Outwards", 1)
        .add("DLA - Central Attractor", 1)
        .add("DLA - Insectoid", 1)
        .add("Voronoi Map Builder", 1)
}
