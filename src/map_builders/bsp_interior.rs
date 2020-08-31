use rltk::RandomNumberGenerator;

use super::common::*;
use super::map::*;
use super::{BuilderSettings, MapBuilder};
use super::rect::Rect;

const MIN_ROOM_SIZE : i32 = 8;

#[derive(Clone)]
pub struct BSPInteriorBuilder {
    pub map: Map,
    pub rects: Vec<Rect>,
}

impl MapBuilder for BSPInteriorBuilder {
    fn new(new_depth: i32) -> Box<dyn MapBuilder> {
        Box::new(BSPInteriorBuilder { map: Map::new(new_depth), rects: Vec::new()})
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
        self.rects.push( Rect::new(1, 1, self.map.width-2, self.map.height-2) ); // Start with a single map-sized rectangle
        let first_room = self.rects[0];
        self.add_subrects(first_room, &mut rng); // Divide the first room

        let rooms = self.rects.clone();
        for r in rooms.iter() {
            let room = *r;
            //room.x2 -= 1;
            //room.y2 -= 1;
            self.map.rooms.push(room);
            for y in room.y1 .. room.y2 {
                for x in room.x1 .. room.x2 {
                    let idx = xy_idx(x, y);
                    if idx > 0 && idx < ((self.map.width * self.map.height)-1) as usize {
                        self.map.tiles[idx] = TileType::Floor;
                    }
                }
            }
            self.map.take_snapshot();
        }

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

        // Convert to regions for spawners
        self.map.convert_rooms_to_regions();

        // Don't forget the downstairs
        let downstairs = self.map.rooms[self.map.rooms.len()-1].center();
        let stairs_idx = xy_idx(downstairs.0, downstairs.1);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        // Same for the upstairs
        self.map.upstairs = self.map.rooms[self.map.rooms.len()-1].center();
        let stairs_idx = xy_idx(self.map.upstairs.0, self.map.upstairs.1);
        self.map.tiles[stairs_idx] = TileType::UpStairs;

        self.map.clone()
    }
}

impl BSPInteriorBuilder {
    fn add_subrects(&mut self, rect : Rect, rng : &mut RandomNumberGenerator) {
        // Remove the last rect from the list
        if !self.rects.is_empty() {
            self.rects.remove(self.rects.len() - 1);
        }
    
        // Calculate boundaries
        let width  = rect.x2 - rect.x1;
        let height = rect.y2 - rect.y1;
        let half_width = width / 2;
        let half_height = height / 2;
    
        let split = rng.roll_dice(1, 4);
    
        if split <= 2 {
            // Horizontal split
            let h1 = Rect::new( rect.x1, rect.y1, half_width-1, height );
            self.rects.push( h1 );
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h1, rng); }
            let h2 = Rect::new( rect.x1 + half_width, rect.y1, half_width, height );
            self.rects.push( h2 );
            if half_width > MIN_ROOM_SIZE { self.add_subrects(h2, rng); }
        } else {
            // Vertical split
            let v1 = Rect::new( rect.x1, rect.y1, width, half_height-1 );
            self.rects.push(v1);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v1, rng); }
            let v2 = Rect::new( rect.x1, rect.y1 + half_height, width, half_height );
            self.rects.push(v2);
            if half_height > MIN_ROOM_SIZE { self.add_subrects(v2, rng); }
        }

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