use rltk::RandomNumberGenerator;

use super::common::*;
use super::map::*;
use super::{BuilderSettings, MapBuilder};

use crate::spawner::components::Position;

#[derive(Clone)]
pub struct CellularAutomataBuilder {
    pub map: Map,
}

impl MapBuilder for CellularAutomataBuilder {
    fn new(new_depth: i32) -> Box<dyn MapBuilder> {
        Box::new(CellularAutomataBuilder { map: Map::new(new_depth) })
    }

    #[allow(unused_variables)]
    fn with_settings (&mut self, settings: BuilderSettings) -> Box<dyn MapBuilder> {
        return Box::new(self.clone());
    }
    
    #[allow(unused_variables)]
    fn add(&mut self, map: Map) -> Box<dyn MapBuilder> {
        return Box::new(self.to_owned());
    }

    fn build(&mut self) -> Map {
        let mut rng = RandomNumberGenerator::new();

        // First we completely randomize the map, setting 55% of it to be floor.
        for y in 1..self.map.height-1 {
            for x in 1..self.map.width-1 {
                let roll = rng.roll_dice(1, 100);
                let idx = xy_idx(x, y);
                if roll > 55 { self.map.tiles[idx] = TileType::Floor } 
                else { self.map.tiles[idx] = TileType::Wall }
            }
        }
        self.map.take_snapshot();

        // Now we iteratively apply cellular automata rules
        for _i in 0..15 {
            let mut newtiles = self.map.tiles.clone();

            for y in 1..self.map.height-1 {
                for x in 1..self.map.width-1 {
                    let idx = xy_idx(x, y);
                    let mut neighbors = 0;
                    if self.map.tiles[idx - 1] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[idx + 1] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[idx - self.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[idx + self.map.width as usize] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[idx - (self.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[idx - (self.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[idx + (self.map.width as usize - 1)] == TileType::Wall { neighbors += 1; }
                    if self.map.tiles[idx + (self.map.width as usize + 1)] == TileType::Wall { neighbors += 1; }

                    if neighbors > 4 || neighbors == 0 {
                        newtiles[idx] = TileType::Wall;
                    }
                    else {
                        newtiles[idx] = TileType::Floor;
                    }
                }
            }

            self.map.tiles = newtiles.clone();
            self.map.take_snapshot();
        }

        // Now we build a noise map for use in spawning entities later
        let mut noise = rltk::FastNoise::seeded(rng.roll_dice(1, 65536) as u64);
        noise.set_noise_type(rltk::NoiseType::Cellular);
        noise.set_frequency(0.08);
        noise.set_cellular_distance_function(rltk::CellularDistanceFunction::Manhattan);

        for y in 1 .. self.map.height-1 {
            for x in 1 .. self.map.width-1 {
                let idx = xy_idx(x, y);
                if self.map.tiles[idx] == TileType::Floor {
                    let cell_value_f = noise.get_noise(x as f32, y as f32) * 10240.0;
                    let cell_value = cell_value_f as i32;

                    if self.map.regions.contains_key(&cell_value) {
                        self.map.regions.get_mut(&cell_value).unwrap().push(idx);
                    } else {
                        self.map.regions.insert(cell_value, vec![idx]);
                    }
                }
            }
        }

        // Find a starting point; start at the middle and walk left until we find an open tile
        let mut starting_position = Position::new(self.map.width / 2, self.map.height / 2);
        let mut start_idx = xy_idx(starting_position.get_x(), starting_position.get_y());
        while self.map.tiles[start_idx] != TileType::Floor {
            starting_position.set_x(starting_position.get_x() - 1);
            start_idx = xy_idx(starting_position.get_x(), starting_position.get_y());
        }
        self.map.upstairs = (starting_position.get_x(), starting_position.get_y());
        self.map.tiles[start_idx] = TileType::UpStairs;

        // Find all tiles we can reach from the starting point
        let map_starts : Vec<usize> = vec![start_idx];
        let dijkstra_map = rltk::DijkstraMap::new(self.map.width, self.map.height, &map_starts , &self.map, 200.0);
        let mut exit_tile = (0, 0.0f32);
        for (i, tile) in self.map.tiles.iter_mut().enumerate() {
            if *tile == TileType::Floor {
                let distance_to_start = dijkstra_map.map[i];
                // We can't get to this tile - so we'll make it a wall
                if distance_to_start == std::f32::MAX {
                    *tile = TileType::Wall;
                } else {
                    // If it is further away than our current exit candidate, move the exit
                    if distance_to_start > exit_tile.1 {
                        exit_tile.0 = i;
                        exit_tile.1 = distance_to_start;
                    }
                }
            }
        }
        self.map.take_snapshot();

        self.map.tiles[exit_tile.0] = TileType::DownStairs;
        self.map.take_snapshot();

        self.map.clone()
    }
}

impl CellularAutomataBuilder {

}