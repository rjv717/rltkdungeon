use rltk::{FontCharType, RGB};
use specs::*;
use specs_derive::*;
use serde::{Serialize, Deserialize};
use specs::saveload::{MarkedBuilder, SimpleMarker};

use crate::components::SerializeMe;
use crate::spawner::components::*;

pub enum MonType{
    Orc,
    Goblin,
}

#[derive(Component, Serialize, Deserialize, Copy, Clone, Debug)]
pub struct Monster {}

impl Monster {
    #[allow(dead_code)]
    pub fn new(ecs: &mut World, x: i32, y: i32, mon: MonType) {
        let glyph: FontCharType;
        let name: String;
        let hp: i32;
        let hp_max: i32;
        let attack: i32;
        let defense: i32;
        match mon {
            MonType::Orc => {
                glyph = rltk::to_cp437('o');
                name = "Orc".into();
                hp = 18;
                hp_max = 18;
                attack = 4;
                defense = 1;
            },
            MonType::Goblin => {
                glyph = rltk::to_cp437('g');
                name = "Goblin".into();
                hp = 16;
                hp_max = 16;
                attack = 4;
                defense = 2;
            }
        }

        ecs.create_entity()
            .with(Actor {})
            .with(Monster {})
            .with(Position::new(x, y))
            .with(Renderable {
                glyph,
                fg: RGB::named(rltk::YELLOW),
                bg: RGB::named(rltk::BLACK),
                render_order: 2,
            })
            .with(Name { name } )
            .with(BlocksTile {} )
            .with(CombatStats { hp, hp_max, attack, defense, is_dead: false})
            .with(Viewshed {
                visible_tiles: Vec::new(),
                range: 8,
                dirty: true,
            })
            .marked::<SimpleMarker<SerializeMe>>()
            .build();
    }
}
