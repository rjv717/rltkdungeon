use rltk::RandomNumberGenerator;

mod common;
mod constraints;
mod solver;

use super::common::*;
use super::map::*;
use super::{BuilderSettings, MapBuilder};
use crate::spawner::components::Position;
use constraints::*;
use common::*;
use solver::*;

#[derive(Clone)]
pub struct WaveFunctionCollapseBuilder {
    pub map: Map,
}

impl WaveFunctionCollapseBuilder {

    fn render_tile_gallery(&mut self, patterns: &[MapChunk], chunk_size: i32) {
        self.map = Map::new(0);
        let mut counter = 0;
        let mut x = 1;
        let mut y = 1;
        while counter < patterns.len() {
            render_pattern_to_map(&mut self.map, &patterns[counter], chunk_size, x, y);
            x += chunk_size + 1;
            if x + chunk_size > self.map.width {
                // Move to the next row
                x = 1;
                y += chunk_size + 1;
                if y + chunk_size > self.map.height {
                    // Move to the next page
                    self.map.take_snapshot();
                    self.map = Map::new(0);
                    x = 1;
                    y = 1;
                }
            }
            counter += 1;
        }
        self.map.take_snapshot();
    }
}

impl MapBuilder for WaveFunctionCollapseBuilder {
    fn build(&mut self) -> Map {
        let mut rng = RandomNumberGenerator::new();

        const CHUNK_SIZE :i32 = 8;

        for t in self.map.tiles.iter_mut() {
            if *t == TileType::DownStairs { *t = TileType::Floor; }
            if *t == TileType::UpStairs { *t = TileType::Floor; }
        }
        self.map.take_snapshot();

        let patterns = build_patterns(&self.map, CHUNK_SIZE, true, true);
        let constraints = patterns_to_constraints(patterns, CHUNK_SIZE);
        self.render_tile_gallery(&constraints, CHUNK_SIZE);

        self.map = Map::new(self.map.depth);
        loop {
            let mut solver = Solver::new(constraints.clone(), CHUNK_SIZE, &self.map);
            while !solver.iteration(&mut self.map, &mut rng) {
                self.map.take_snapshot();
            }
            self.map.take_snapshot();
            if solver.possible { break; } // If it has hit an impossible condition, try again
        }

        make_boundary_walls(&mut self.map);

        // Find a starting point; start at the middle and walk left until we find an open tile
        let mut starting_position = Position::new( self.map.width / 2, self.map.height / 2 );
        let mut start_idx = xy_idx(starting_position.get_x(), starting_position.get_y());
        while self.map.tiles[start_idx] != TileType::Floor {
            starting_position.set_x(starting_position.get_x() - 1);
            start_idx = xy_idx(starting_position.get_x(), starting_position.get_y());
        }
        self.map.take_snapshot();

        // Find all tiles we can reach from the starting point
        let exit_tile = remove_unreachable_areas_returning_most_distant(&mut self.map, start_idx);
        self.map.take_snapshot();

        // Place the stairs
        self.map.tiles[start_idx] = TileType::UpStairs;
        self.map.upstairs = (starting_position.get_x(), starting_position.get_y());
        self.map.tiles[exit_tile] = TileType::DownStairs;
        self.map.take_snapshot();

        // Now we build a noise map for use in spawning entities later
        self.map.regions = generate_voronoi_spawn_regions(&self.map, &mut rng);
        self.map.clone()
    }

    #[allow(unused_variables)]
    fn with_settings(&mut self, settings: BuilderSettings) -> Box<dyn MapBuilder> {
        return Box::new(self.clone());
    }

    fn add(&mut self, map: Map) -> Box<dyn MapBuilder> {
        self.map = map.to_owned();
        Box::new(self.to_owned())
    }

    fn new(new_depth: i32) -> Box<dyn MapBuilder> {
        Box::new(WaveFunctionCollapseBuilder {
            map: Map::new(new_depth),

        })
    }
}
