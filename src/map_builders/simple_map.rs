use rltk::RandomNumberGenerator;

use super::common::*;
use super::map::*;
use super::{BuilderSettings, MapBuilder};
use super::rect::Rect;

#[derive(Clone)]
pub struct SimpleMapBuilder {
    pub map: Map,
}

impl SimpleMapBuilder {
    
}

impl MapBuilder for SimpleMapBuilder {
    fn build(&mut self) -> Map {
        rooms_and_corridors(&mut self.map);
        
        self.map.convert_rooms_to_regions();

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
        Box::new(SimpleMapBuilder { map: Map::new (new_depth)})
    }
}

fn rooms_and_corridors(map : &mut Map) {
    const MAX_ROOMS : i32 = 30;
    const MIN_SIZE : i32 = 6;
    const MAX_SIZE : i32 = 10;

    let mut rng = RandomNumberGenerator::new();

    for _i in 0..MAX_ROOMS {
        let w = rng.range(MIN_SIZE, MAX_SIZE);
        let h = rng.range(MIN_SIZE, MAX_SIZE);
        let x = rng.roll_dice(1, map.width - w - 2);
        let y = rng.roll_dice(1, map.height - h - 2);
        let new_room = Rect::new(x, y, w, h);
        let mut ok = true;
        for other_room in map.rooms.iter() {
            if new_room.intersect(other_room) { ok = false }
        }
        if ok {
            apply_room_to_map(&new_room, map);
            map.take_snapshot();

            if !map.rooms.is_empty() {
                let (new_x, new_y) = new_room.center();
                let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                if rng.range(0,2) == 1 {
                    apply_horizontal_tunnel(map, prev_x, new_x, prev_y);
                    apply_vertical_tunnel(map, prev_y, new_y, new_x);
                    map.take_snapshot();
                } else {
                    apply_vertical_tunnel(map, prev_y, new_y, prev_x);
                    apply_horizontal_tunnel(map, prev_x, new_x, new_y);
                    map.take_snapshot();
                }
            }

            map.rooms.push(new_room);
        }
    }

    map.populate_blocked();
    
    map.upstairs = map.rooms[0].center();
    let upstairs_idx = xy_idx(map.upstairs.0, map.upstairs.1);
    map.tiles[upstairs_idx] = TileType::UpStairs;
    let stairs_position = map.rooms[map.rooms.len()-1].center();
    let stairs_idx = xy_idx(stairs_position.0, stairs_position.1);
    map.tiles[stairs_idx] = TileType::DownStairs;
}