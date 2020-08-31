use rltk::RandomNumberGenerator;

use super::common::*;
use super::map::*;
use super::{BuilderSettings, DLAAlgorithm, MapBuilder}; //, PrefabMode};

use crate::spawner::components::Position;

#[derive(Clone)]
pub struct DLABuilder {
    pub map: Map,
    pub settings: BuilderSettings,
}

impl MapBuilder for DLABuilder {
    fn new(new_depth: i32) -> Box<dyn MapBuilder> {
        Box::new(DLABuilder {
            map: Map::new(new_depth),
            settings: BuilderSettings {
                spawn_mode: None,
                lifetime: None,
                floor_percent: Some(0.25),
                algorithm: Some(DLAAlgorithm::WalkInwards),
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

        // Carve a starting seed
        let starting_position = Position::new(self.map.width / 2, self.map.height / 2);
        let start_idx = xy_idx(starting_position.get_x(), starting_position.get_y());
        self.map.take_snapshot();
        self.map.tiles[start_idx] = TileType::Floor;
        self.map.tiles[start_idx-1] = TileType::Floor;
        self.map.tiles[start_idx+1] = TileType::Floor;
        self.map.tiles[start_idx-self.map.width as usize] = TileType::Floor;
        self.map.tiles[start_idx+self.map.width as usize] = TileType::Floor;

        // Random walker
        let total_tiles = self.map.width * self.map.height;
        let desired_floor_tiles = (self.settings.floor_percent.unwrap() * total_tiles as f32) as usize;
        let mut floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        while floor_tile_count  < desired_floor_tiles {

            match self.settings.algorithm.unwrap() {
                DLAAlgorithm::WalkInwards => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = xy_idx(digger_x, digger_y);
                    while self.map.tiles[digger_idx] == TileType::Wall {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } }
                            2 => { if digger_x < self.map.width-2 { digger_x += 1; } }
                            3 => { if digger_y > 2 { digger_y -=1; } }
                            _ => { if digger_y < self.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = xy_idx(digger_x, digger_y);
                    }
                    paint(&mut self.map, self.settings.symmetry.unwrap(), self.settings.brush_size.unwrap(), prev_x, prev_y);
                }
                DLAAlgorithm::WalkOutwards => {
                    let mut digger_x = starting_position.get_x();
                    let mut digger_y = starting_position.get_y();
                    let mut digger_idx = xy_idx(digger_x, digger_y);
                    while self.map.tiles[digger_idx] == TileType::Floor {
                        let stagger_direction = rng.roll_dice(1, 4);
                        match stagger_direction {
                            1 => { if digger_x > 2 { digger_x -= 1; } }
                            2 => { if digger_x < self.map.width-2 { digger_x += 1; } }
                            3 => { if digger_y > 2 { digger_y -=1; } }
                            _ => { if digger_y < self.map.height-2 { digger_y += 1; } }
                        }
                        digger_idx = xy_idx(digger_x, digger_y);
                    }
                    paint(&mut self.map, self.settings.symmetry.unwrap(), self.settings.brush_size.unwrap(), digger_x, digger_y);
                }
                DLAAlgorithm::CentralAttractor => {
                    let mut digger_x = rng.roll_dice(1, self.map.width - 3) + 1;
                    let mut digger_y = rng.roll_dice(1, self.map.height - 3) + 1;
                    let mut prev_x = digger_x;
                    let mut prev_y = digger_y;
                    let mut digger_idx = xy_idx(digger_x, digger_y);
                
                    let mut path = rltk::line2d(
                        rltk::LineAlg::Bresenham, 
                        rltk::Point::new( digger_x, digger_y ), 
                        rltk::Point::new( starting_position.get_x(), starting_position.get_y() )
                    );
                
                    while self.map.tiles[digger_idx] == TileType::Wall && !path.is_empty() {
                        prev_x = digger_x;
                        prev_y = digger_y;
                        digger_x = path[0].x;
                        digger_y = path[0].y;
                        path.remove(0);
                        digger_idx = xy_idx(digger_x, digger_y);
                    }
                    paint(&mut self.map, self.settings.symmetry.unwrap(), self.settings.brush_size.unwrap(), prev_x, prev_y);
                }
            }
            self.map.take_snapshot();

            floor_tile_count = self.map.tiles.iter().filter(|a| **a == TileType::Floor).count();
        }

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

impl DLABuilder {
    
}
