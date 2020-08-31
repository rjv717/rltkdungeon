use rltk::RandomNumberGenerator;

use super::common::*;
use super::map::*;
use super::{BuilderSettings, MapBuilder};
use super::rect::Rect;

#[derive(Clone)]
pub struct BSPDungeonBuilder {
    pub map: Map,
    pub rects: Vec<Rect>,
}

impl MapBuilder for BSPDungeonBuilder {

    fn new(new_depth: i32) -> Box<dyn MapBuilder> {
        Box::new(BSPDungeonBuilder { map: Map::new(new_depth), rects: Vec::new()})
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

        self.rects.clear();
        self.rects.push( Rect::new(2, 2, self.map.width-5, self.map.height-5));
        let first_room = self.rects[0];
        self.add_subrects(first_room); // Divide the first room

        // Up to 240 times, we get a random rectangle and divide it. If its possible to squeeze a
        // room in there, we place it and add it to the rooms list.
        let mut n_rooms = 0;
        while n_rooms < 240 {
            let rect = self.get_random_rect(&mut rng);
            let candidate = self.get_random_sub_rect(rect, &mut rng);

            if self.is_possible(candidate) {
                apply_room_to_map(&candidate, &mut self.map);
                self.map.rooms.push(candidate);
                self.add_subrects(rect);
                self.map.take_snapshot();
            }

            n_rooms += 1;
        }
        self.map.rooms.sort_by(|a,b| a.x1.cmp(&b.x1) );

        // Now we want corridors
        for i in 0..self.map.rooms.len()-1 {
            let room = self.map.rooms[i];
            let next_room = self.map.rooms[i+1];
            let start_x = room.x1 + (rng.roll_dice(1, i32::abs(room.x1 - room.x2))-1);
            let start_y = room.y1 + (rng.roll_dice(1, i32::abs(room.y1 - room.y2))-1);
            let end_x = next_room.x1 + (rng.roll_dice(1, i32::abs(next_room.x1 - next_room.x2))-1);
            let end_y = next_room.y1 + (rng.roll_dice(1, i32::abs(next_room.y1 - next_room.y2))-1);
            self.draw_corridor(start_x, start_y, end_x, end_y);
            self.map.take_snapshot();
        }

        // Convert to regions for spawners.
        self.map.convert_rooms_to_regions();

        // Don't forget the stairs
        let down_stairs = self.map.rooms[self.map.rooms.len()-1].center();
        let stairs_idx = xy_idx(down_stairs.0, down_stairs.1);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        self.map.upstairs = self.map.rooms[0].center();
        let stairs_idx = xy_idx(self.map.upstairs.0, self.map.upstairs.1);
        self.map.tiles[stairs_idx] = TileType::UpStairs;

        self.map.clone()
    }
}

impl BSPDungeonBuilder {
    fn add_subrects(&mut self, rect : Rect) {
        let width = i32::abs(rect.x1 - rect.x2);
        let height = i32::abs(rect.y1 - rect.y2);
        let half_width = i32::max(width / 2, 1);
        let half_height = i32::max(height / 2, 1);

        self.rects.push(Rect::new( rect.x1, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1, rect.y1 + half_height, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1, half_width, half_height ));
        self.rects.push(Rect::new( rect.x1 + half_width, rect.y1 + half_height, half_width, half_height ));
    }

    fn get_random_rect(&mut self, rng : &mut RandomNumberGenerator) -> Rect {
        if self.rects.len() == 1 { return self.rects[0]; }
        let idx = (rng.roll_dice(1, self.rects.len() as i32)-1) as usize;
        self.rects[idx]
    }

    fn get_random_sub_rect(&self, rect : Rect, rng : &mut RandomNumberGenerator) -> Rect {
        let mut result = rect;
        let rect_width = i32::abs(rect.x1 - rect.x2);
        let rect_height = i32::abs(rect.y1 - rect.y2);
    
        let w = i32::max(3, rng.roll_dice(1, i32::min(rect_width, 10))-1) + 1;
        let h = i32::max(3, rng.roll_dice(1, i32::min(rect_height, 10))-1) + 1;
    
        result.x1 += rng.roll_dice(1, 6)-1;
        result.y1 += rng.roll_dice(1, 6)-1;
        result.x2 = result.x1 + w;
        result.y2 = result.y1 + h;
    
        result
    }

    fn is_possible(&self, rect : Rect) -> bool {
        let mut expanded = rect;
        expanded.x1 -= 2;
        expanded.x2 += 2;
        expanded.y1 -= 2;
        expanded.y2 += 2;
    
        let mut can_build = true;
    
        for y in expanded.y1 ..= expanded.y2 {
            for x in expanded.x1 ..= expanded.x2 {
                if x > self.map.width-2 { can_build = false; }
                if y > self.map.height-2 { can_build = false; }
                if x < 1 { can_build = false; }
                if y < 1 { can_build = false; }
                if can_build {
                    let idx = xy_idx(x, y);
                    if self.map.tiles[idx] != TileType::Wall { 
                        can_build = false; 
                    }
                }
            }
        }
    
        can_build
    }

    fn draw_corridor(&mut self, x1:i32, y1:i32, x2:i32, y2:i32) {
        let mut x = x1;
        let mut y = y1;
    
        while x != x2 || y != y2 {
            if x < x2 {
                x += 1;
            } else if x > x2 {
                x -= 1;
            } else if y < y2 {
                y += 1;
            } else if y > y2 {
                y -= 1;
            }
    
            let idx = xy_idx(x, y);
            self.map.tiles[idx] = TileType::Floor;
        }
    }
}