use rltk::RandomNumberGenerator;

use super::common::*;
use super::map::*;
use super::{BuilderSettings, DrunkSpawnMode, MapBuilder}; //, PrefabMode};

use crate::spawner::components::Position;

#[derive(Clone)]
pub struct DrunkardsWalkBuilder {
    pub map: Map,
    pub settings: BuilderSettings,
}

impl MapBuilder for DrunkardsWalkBuilder {
    fn new(new_depth: i32) -> Box<dyn MapBuilder> {
        Box::new(DrunkardsWalkBuilder {
            map: Map::new(new_depth),
            settings: BuilderSettings {
                spawn_mode: Some(DrunkSpawnMode::StartingPoint),
                lifetime: Some(400),
                floor_percent: Some(0.5),
                algorithm: None,
                symmetry: Some(Symmetry::None),
                brush_size: Some(1),
                //mode: PrefabMode::None,
            },
        })
    }

    fn with_settings (&mut self, settings: BuilderSettings) -> Box<dyn MapBuilder> {
        self.settings = settings;

        return Box::new(self.clone());
    }

    #[allow(unused_variables)]
    fn add(&mut self, map: Map) -> Box<dyn MapBuilder> {
        return Box::new(self.to_owned());
    }

    fn build(&mut self) -> Map {
        let mut rng = RandomNumberGenerator::new();

        // Set a central starting point
        let starting_position = Position::new(self.map.width / 2, self.map.height / 2);
        let start_idx = xy_idx(starting_position.get_x(), starting_position.get_y());
        self.map.tiles[start_idx] = TileType::Floor;

        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.settings.floor_percent.unwrap() * total_tiles as f32) as usize;
        
        let mut floor_tile_count = self
            .map
            .tiles
            .iter()
            .filter(|a| **a == TileType::Floor)
            .count();
        let mut digger_count = 0;
        let mut active_digger_count = 0;

        while floor_tile_count  < desired_floor_tiles {
            let mut did_something = false;
            let mut drunk_x;
            let mut drunk_y;
            match self.settings.spawn_mode.unwrap() {
                DrunkSpawnMode::StartingPoint => {
                    drunk_x = starting_position.get_x();
                    drunk_y = starting_position.get_y();
                }
                DrunkSpawnMode::Random => {
                    if digger_count == 0 {
                        drunk_x = starting_position.get_x();
                        drunk_y = starting_position.get_y();
                    } else {
                        drunk_x = rng.roll_dice(1, self.map.width - 3) + 1;
                        drunk_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    }
                }
            }
            let mut drunk_life = self.settings.lifetime.unwrap();

            while drunk_life > 0 {
                let drunk_idx = xy_idx(drunk_x, drunk_y);
                if self.map.tiles[drunk_idx] == TileType::Wall {
                    did_something = true;
                }
                paint(&mut self.map, self.settings.symmetry.unwrap(), self.settings.brush_size.unwrap(), drunk_x, drunk_y);
                self.map.tiles[drunk_idx] = TileType::DownStairs;

                let stagger_direction = rng.roll_dice(1, 4);
                match stagger_direction {
                    1 => {
                        if drunk_x > 2 {
                            drunk_x -= 1;
                        }
                    }
                    2 => {
                        if drunk_x < self.map.width - 2 {
                            drunk_x += 1;
                        }
                    }
                    3 => {
                        if drunk_y > 2 {
                            drunk_y -= 1;
                        }
                    }
                    _ => {
                        if drunk_y < self.map.height - 2 {
                            drunk_y += 1;
                        }
                    }
                }

                drunk_life -= 1;
            }
            if did_something {
                self.map.take_snapshot();
                active_digger_count += 1;
            }

            digger_count += 1;
            for t in self.map.tiles.iter_mut() {
                if *t == TileType::DownStairs {
                    *t = TileType::Floor;
                }
            }
            floor_tile_count = self
                .map
                .tiles
                .iter()
                .filter(|a| **a == TileType::Floor)
                .count();
        }
        rltk::console::log(format!(
            "{} dwarves gave up their sobriety, of whom {} actually found a wall.",
            digger_count, active_digger_count
        ));

        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.map.take_snapshot();

        // Place the stairs
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.map.upstairs = (starting_position.get_x(), starting_position.get_y());
        let up_idx = xy_idx(self.map.upstairs.0, self.map.upstairs.1);
        self.map.tiles[up_idx] = TileType::UpStairs;
        self.map.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.map.regions = generate_voronoi_spawn_regions(&self.map, &mut rng);

        self.map.clone()
    }
}

impl DrunkardsWalkBuilder {}
