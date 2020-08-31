use rltk::{Algorithm2D, BaseMap, FontCharType, Point, Rltk, RGB};
use serde::{Deserialize, Serialize};
use specs::*;
use std::collections::{HashMap, HashSet};

use crate::SHOW_MAPGEN_VISUALIZER;
use super::common::*;
use super::rect::Rect;

/// TileType enum to declare the structural components of the Map.
#[derive(Hash, Eq, PartialEq, Serialize, Deserialize, Copy, Clone)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    UpStairs,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub rooms: Vec<Rect>,
    pub regions: HashMap<i32, Vec<usize>>,
    pub blocked: Vec<bool>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub depth: i32,
    pub blood_stains: HashSet<usize>,
    pub magic_map: Vec<bool>,
    pub upstairs: (i32, i32),

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
    pub history: Vec<Vec<TileType>>,
    pub history_count: usize,
}

impl Map {

    pub fn get_upstairs(&self) -> (i32, i32) {
        self.upstairs
    }

    pub fn is_blocked(&self, x: i32, y: i32) -> bool {
        let idx = xy_idx(x, y);
        self.blocked[idx]
    }

    pub fn is_magic_mapped(&self, x: i32, y: i32) -> bool {
        let idx = xy_idx(x, y);
        return self.magic_map[idx];
    }
    
    pub fn is_tile_revealable(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width-2 || y < 1 || y > self.height-2 {
            return false;
        }
        for delta_x in -1..=1 {
            for delta_y in -1..=1 {
                if !self.is_blocked(x+delta_x, y+delta_y) {
                    return true;
                }
            }
        }
        false
    }

    pub fn reveal_me (&mut self, x: i32, y: i32) {
        let idx = xy_idx(x, y);
        if self. is_tile_revealable(x, y) {
            self.revealed_tiles[idx] = true;
        }
        self.magic_map[idx] = true;
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn convert_rooms_to_regions (&mut self) {
        let mut region = Vec::new();
        for (index, room) in self.rooms.iter().enumerate() {
            for x in room.x1..=room.x2 {
                for y in room.y1..=room.y2 {
                    let idx = xy_idx(x, y);
                    region.push(idx);
                }
            }
            self.regions.insert(index as i32, region.clone());
        }
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            match tile {
                TileType::Wall => {
                    self.blocked[i as usize] = true;
                }
                _ => {
                    self.blocked[i as usize] = false;
                }
            }
        }
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = xy_idx(x, y);
        // !self.blocked[idx]
        self.tiles[idx] != TileType::Wall
    }

    pub fn take_snapshot (&mut self) {
        if SHOW_MAPGEN_VISUALIZER {
            self.history.push(self.tiles.clone());
        }
    }

    pub fn get_snapshot (&mut self) -> Option<usize> {
        if SHOW_MAPGEN_VISUALIZER {
            if self.history_count < self.history.len() {
                self.history_count += 1;
                return Some(self.history_count);
            } else {
                self.history_count = 0;
                return None;
            }
        }
        return None;
    }

    pub fn draw_map(&self, ident: usize, ctx: &mut Rltk) {
        let tiles;
        if ident == 0 {
            tiles = &self.tiles;
        } else {
            if ident - 1 > self.history.len() { return; }
            tiles = &self.history[ident - 1];
        }
        let mut y = 0;
        let mut x = 0;
        for (idx, tile) in tiles.iter().enumerate() {
            // Render a tile depending upon the tile type
            if self.revealed_tiles[idx] || ident > 0 {
                let mut fg: RGB;
                let mut bg = RGB::from_f32(0.0, 0.0, 0.0);
                let glyph: FontCharType;
                match tile {
                    TileType::Floor => {
                        fg = RGB::from_f32(0.5, 0.5, 0.5);
                        glyph = rltk::to_cp437('·');
                    }
                    TileType::Wall => {
                        fg = RGB::from_f32(0.0, 1.0, 0.0);
                        if ident == 0 {
                            glyph = self.wall_glyph(x, y);
                        } else {
                            glyph = rltk::to_cp437('#');
                        }
                    }
                    TileType::DownStairs => {
                        fg = RGB::from_f32(0.0, 0.75, 0.0);
                        glyph = rltk::to_cp437('>');
                    }
                    TileType::UpStairs => {
                        fg = RGB::from_f32(0.0, 0.75, 0.0);
                        glyph = rltk::to_cp437('<');
                    }
                }
                if ident == 0 {
                    if self.blood_stains.contains(&idx) {
                        bg = RGB::from_f32(0.75, 0., 0.);
                    }
                    if !self.visible_tiles[idx] {
                        fg = fg.to_greyscale();
                        bg = RGB::from_f32(0.0, 0.0, 0.0);
                    }
                }
                ctx.set(x, y, fg, bg, glyph);
            }
            // Move the coordinates
            x += 1;
            if x > 79 {
                x = 0;
                y += 1;
            }
        }
    }

    fn wall_glyph(&self, x: i32, y: i32) -> rltk::FontCharType {
        let mut mask: u8 = 0;
        if x < 1 as i32 {
            mask += 1;
        }
        if x > self.width - 2 as i32 {
            mask += 2;
        }
        if y < 1 as i32 {
            mask += 4;
        }
        if y > self.height - 2 as i32 {
            mask += 8;
        }

        if x >= 0 && x <= self.width - 2 && y >= 1 && y <= self.width - 2 {
            if self.is_revealed_and_wall(x, y - 1) {
                mask += 1;
            }
            if self.is_revealed_and_wall(x, y + 1) {
                mask += 2;
            }
            if self.is_revealed_and_wall(x - 1, y) {
                mask += 4;
            }
            if self.is_revealed_and_wall(x + 1, y) {
                mask += 8;
            }
        }
        match mask {
            0 => 9,    // Pillar because we can't see neighbors
            1 => 186,  // Wall only to the north
            2 => 186,  // Wall only to the south
            3 => 186,  // Wall to the north and south
            4 => 205,  // Wall only to the west
            5 => 188,  // Wall to the north and west
            6 => 187,  // Wall to the south and west
            7 => 185,  // Wall to the north, south and west
            8 => 205,  // Wall only to the east
            9 => 200,  // Wall to the north and east
            10 => 201, // Wall to the south and east
            11 => 204, // Wall to the north, south and east
            12 => 205, // Wall to the east and west
            13 => 202, // Wall to the east, west, and south
            14 => 203, // Wall to the east, west, and north
            15 => 206, // ╬ Wall on all sides
            _ => 35,   // We missed one?
        }
    }

    fn is_revealed_and_wall(&self, x: i32, y: i32) -> bool {
        let idx = xy_idx(x, y);
        if idx > (self.width * self.height) as usize {
            return false;
        }
        self.tiles[idx] == TileType::Wall && self.revealed_tiles[idx]
    }

    pub fn new(new_depth: i32) -> Map {

        let width: i32 = 80;
        let height: i32 = 43;
        let mut map = Map {
            tiles: vec![TileType::Wall; (width * height) as usize],
            rooms: Vec::new(),
            regions: HashMap::new(),
            blocked: vec![false; (width * height) as usize],
            width,
            height,
            revealed_tiles: vec![false; (width * height) as usize],
            visible_tiles: vec![false; (width * height) as usize],
            tile_content: vec![Vec::new(); (width * height) as usize],
            depth: new_depth,
            blood_stains: HashSet::new(),
            magic_map: vec![false; (width * height) as usize],
            upstairs: (0, 0),
            history: Vec::new(),
            history_count: 0,
        };

        make_boundary_walls(&mut map);

        map
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;
        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };
        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}
