use rltk::RandomNumberGenerator;
use specs::*;


pub mod components;
pub mod item;
pub mod monster;
pub mod player;
pub mod random_table;

use crate::map_builders::map::Map;
use crate::spawner::monster::MonType;
use crate::spawner::random_table::*;
use crate::spawner::{item::Item, monster::Monster, player::Player};

pub enum SpawnSeed {
    Player,
    Monster,
    Item,
}

pub fn spawn(ecs: &mut World, map: &Map, item: SpawnSeed) {
    match item {
        SpawnSeed::Player => {
            let (x, y) = map.get_upstairs();
            Player::new(ecs, x, y);
        }
        SpawnSeed::Monster => {
            for (_index, region) in map.regions.iter().skip(1) {
                let mut monster_spawn_points: Vec<usize> = Vec::new();
                let monster_name: String;

                // scope to keep the borrow checker happy.
                {
                    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
                    let max_mon = map.regions.len() / 27 + 1;
                    monster_name = monster_table(map).roll(&mut rng);   
                    let num_monsters = rng.roll_dice(1, max_mon as i32) + (map.depth - 1);
                    
                    for _i in 0..num_monsters {
                        let mut added = false;
                        while !added {
                            let rnd_index = rng.roll_dice(1, region.len() as i32) - 1;
                            let idx = region[rnd_index as usize];
                            if !monster_spawn_points.contains(&idx) {
                                monster_spawn_points.push(idx);
                                added = true;
                            }
                        }
                    }
                }
                // Actually spawn the monsters
                {
                    for idx in monster_spawn_points.iter() {
                        let x = (*idx % map.width as usize) as i32;
                        let y = (*idx / map.width as usize) as i32;
                        match monster_name.as_ref() {
                            "Goblin" => goblin(ecs, x, y),
                            "Orc" => orc(ecs, x, y),
                            _ => { }
                        }
                    }
                }
            }
        }
        SpawnSeed::Item => {
            for (_index, region) in map.regions.iter() {
                let number;
                {
                    let mut rng = ecs.write_resource::<RandomNumberGenerator>();
                    number = rng.roll_dice(2, 6);
                }
                let mut chance;
                let mut x;
                let mut y;
                {
                    for _i in 1 ..= number {
                        {
                            let mut rng = ecs.write_resource::<RandomNumberGenerator>();
                            chance = rng.roll_dice(1, 3);
                            let rnd_index = rng.roll_dice(1, region.len() as i32) - 1;
                            let idx = region[rnd_index as usize];
                            x = idx as i32 % map.width;
                            y = idx as i32 / map.width;
                        }
                        
                        if chance < 2 {
                    
                            Item::new(ecs, x, y );
                        }
                    }
                }
            }
        },
    }
}

fn monster_table(map: &Map) -> RandomTable {

    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map.depth)
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    Monster::new(ecs, x, y, MonType::Orc);
}
fn goblin(ecs: &mut World, x: i32, y: i32) {
    Monster::new(ecs, x, y, MonType::Goblin);
}
