//use rltk::RandomNumberGenerator;
//use specs::prelude::*;

pub mod prefab_levels;

use crate::map_builders::common::xy_idx;
use crate::spawner::components::Position;
use super::map::{Map, TileType};
use super::{BuilderSettings, MapBuilder, PrefabMode};

#[derive(Clone)]
pub struct PrefabBuilder {
    map : Map,
    settings: BuilderSettings,
}

#[allow(unused_variables)]
impl MapBuilder for PrefabBuilder {
    fn build(&mut self) -> Map {
        match self.settings.mode {
            PrefabMode::RexLevel{template} => self.load_rex_map(&template),
            PrefabMode::Constant{level} => self.load_ascii_map(&level),
            PrefabMode::None => (),
        }
    
        // Find a starting point; start at the middle and walk left until we find an open tile
        let mut starting_position = Position::new(self.map.width / 2, self.map.height / 2 );
        let mut start_idx = xy_idx(starting_position.get_x(), starting_position.get_y());
        while self.map.tiles[start_idx] != TileType::Floor {
            starting_position.set_x(starting_position.get_x() - 1);
            start_idx = xy_idx(starting_position.get_x(), starting_position.get_y());
        }
        self.map.take_snapshot();

        // TODO: Build Code goes here.

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
        Box::new(PrefabBuilder { map: Map::new (new_depth),settings: BuilderSettings {
            spawn_mode: None,
            lifetime: None,
            floor_percent: None,
            algorithm: None,
            symmetry: None,
            brush_size: None,
            mode: PrefabMode::Constant{level : prefab_levels::WFC_POPULATED},
        },})
    }
}

impl PrefabBuilder {

    fn char_to_map(&mut self, ch : char, idx: usize) -> Position {
        let starting_position: Position;
        match ch {
            ' ' => self.map.tiles[idx] = TileType::Floor,
            '#' => self.map.tiles[idx] = TileType::Wall,
            '@' => {
                let x = idx as i32 % self.map.width;
                let y = idx as i32 / self.map.width;
                self.map.tiles[idx] = TileType::Floor;
                starting_position = Position::new(x as i32, y as i32);
            }
            '>' => self.map.tiles[idx] = TileType::DownStairs,
            'g' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Goblin".to_string()));
            }
            'o' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Orc".to_string()));
            }
            '^' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Bear Trap".to_string()));
            }
            '%' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Rations".to_string()));
            }
            '!' => {
                self.map.tiles[idx] = TileType::Floor;
                self.spawns.push((idx, "Health Potion".to_string()));
            }
            _ => {
                rltk::console::log(format!("Unknown glyph loading map: {}", (ch as u8) as char));
            }
        }
        starting_position
    }

    fn load_rex_map(&mut self, path: &str) {
        let xp_file = rltk::rex::XpFile::from_resource(path).unwrap();
        let starting_position: Position;

        for layer in &xp_file.layers {
            for y in 0..layer.height {
                for x in 0..layer.width {
                    let cell = layer.get(x, y).unwrap();
                    if x < self.map.width as usize && y < self.map.height as usize {
                        let idx = xy_idx(x as i32, y as i32);
                        // We're doing some nasty casting to make it easier to type things like '#' in the match
                        starting_position = self.char_to_map(cell.ch as u8 as char, idx);
                    }
                }
            }
        }
    }
    
    fn load_ascii_map(&mut self, level: &prefab_levels::PrefabLevel) {
        // Start by converting to a vector, with newlines removed
        let starting_position: Position;
        let mut string_vec : Vec<char> = level.template.chars().filter(|a| *a != '\r' && *a !='\n').collect();
        for c in string_vec.iter_mut() { if *c as u8 == 160u8 { *c = ' '; } }
    
        let mut i = 0;
        for ty in 0..level.height {
            for tx in 0..level.width {
                if tx < self.map.width as usize && ty < self.map.height as usize {
                    let idx = xy_idx(tx as i32, ty as i32);
                    starting_position = self.char_to_map(string_vec[i], idx);
                }
                i += 1;
            }
        }
    }
    
}
