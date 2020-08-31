use rltk::RandomNumberGenerator;

use super::common::*;
use super::map::*;
use super::{BuilderSettings, MapBuilder};

#[derive(Clone)]
pub struct VoronoiBuilder {
    pub map: Map,
}

impl VoronoiBuilder {
    
}

impl MapBuilder for VoronoiBuilder {
    fn build(&mut self) -> Map {
        let mut rng = RandomNumberGenerator::new();
        let n_seeds = 64;
        let mut voronoi_seeds : Vec<(usize, rltk::Point)> = Vec::new();

        while voronoi_seeds.len() < n_seeds {
            let vx = rng.roll_dice(1, self.map.width-1);
            let vy = rng.roll_dice(1, self.map.height-1);
            let vidx = xy_idx(vx, vy);
            let candidate = (vidx, rltk::Point::new(vx, vy));
            if !voronoi_seeds.contains(&candidate) {
                voronoi_seeds.push(candidate);
            }
        }

        let mut voronoi_distance = vec![(0, 0.0f32) ; n_seeds];
        let mut voronoi_membership : Vec<i32> = vec![0 ; self.map.width as usize * self.map.height as usize];
        for (i, vid) in voronoi_membership.iter_mut().enumerate() {
            let x = i as i32 % self.map.width;
            let y = i as i32 / self.map.width;

            for (seed, pos) in voronoi_seeds.iter().enumerate() {
                let distance = rltk::DistanceAlg::PythagorasSquared.distance2d(
                    rltk::Point::new(x, y), 
                    pos.1
                );
                voronoi_distance[seed] = (seed, distance);
            }

            voronoi_distance.sort_by(|a,b| a.1.partial_cmp(&b.1).unwrap());

            *vid = voronoi_distance[0].0 as i32;
        }

        for y in 1..self.map.height-1 {
            for x in 1..self.map.width-1 {
                let mut neighbors = 0;
                let my_idx = xy_idx(x, y);
                let my_seed = voronoi_membership[my_idx];
                if voronoi_membership[xy_idx(x-1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[xy_idx(x+1, y)] != my_seed { neighbors += 1; }
                if voronoi_membership[xy_idx(x, y-1)] != my_seed { neighbors += 1; }
                if voronoi_membership[xy_idx(x, y+1)] != my_seed { neighbors += 1; }
        
                if neighbors < 2 {
                    self.map.tiles[my_idx] = TileType::Floor;
                }
            }
            self.map.take_snapshot();
        }  

        self.map.clone()
    }

    #[allow(unused_variables)]
    fn with_settings (&mut self, settings: BuilderSettings) -> Box<dyn MapBuilder> {
        return Box::new(self.clone());
    }

    #[allow(unused_variables)]
    fn add(&mut self, map: Map) -> Box<dyn MapBuilder> {
        return Box::new(self.to_owned());
    }

    fn new(new_depth: i32) -> Box<dyn MapBuilder> {
        Box::new(VoronoiBuilder { map: Map::new (new_depth)})
    }
}

