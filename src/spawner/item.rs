use rltk::{RandomNumberGenerator, *};
use serde::{Deserialize, Serialize};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use specs_derive::*;

use crate::components::SerializeMe;
use crate::map_builders::map::Map;
use crate::spawner::{components::*, random_table::RandomTable};

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Item {}

impl Item {
    pub fn new(ecs: &mut World, x: i32, y: i32) {
        let item_name: String;
        {
            let mut rng = ecs.write_resource::<RandomNumberGenerator>();
            let map = ecs.fetch::<Map>();
            item_name = item_table(&*map).roll(&mut rng);
        }
        match item_name.as_ref() {
            "Healing Potion" => {
                healing_potion(ecs, x, y);
            }
            "Fireball Scroll" => {
                fireball_scroll(ecs, x, y);
            }
            "Confusion Scroll" => {
                confusion_scroll(ecs, x, y);
            }
            "Magic Missile Scroll" => {
                magic_missile_scroll(ecs, x, y);
            }
            "Dagger" => {
                dagger(ecs, x, y);
            }
            "Shield" => {
                shield(ecs, x, y);
            }
            "Longsword" => {
                longsword(ecs, x, y);
            }
            "Tower Shield" => {
                tower_shield(ecs, x, y);
            }
            "Rations" => {
                rations (ecs, x, y);
            }
            "Magic Mapping Scroll" => {
                magic_mapping_scroll(ecs, x, y);
            }
            "Bear Trap" => {
                bear_trap(ecs, x, y);
            }
            _ => {}
        }
    }
}

fn item_table(map: &Map) -> RandomTable {
    RandomTable::new()
        .add("Healing Potion", 7)
        .add("Fireball Scroll", 2)
        .add("Confusion Scroll", 1 + map.depth)
        .add("Magic Missle Scroll", 2)
        .add("Dagger", 1)
        .add("Shield", 1)
        .add("Longsword", map.depth - 1)
        .add("Tower Shield", map.depth - 1)
        .add("Rations", 7)
        .add("Magic Mapping Scroll", map.depth/2 - 1 )
        .add("Bear Trap", 1)
}

/// 
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Potion {}

fn healing_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Item {})
        .with(Name {
            name: "Healing Potion".into(),
        })
        .with(Position::new(x, y))
        .with(Renderable {
            glyph: rltk::to_cp437('|'),
            fg: rltk::RGB::named(rltk::WHITE),
            bg: rltk::RGB::named(rltk::BLACK),
            render_order: 3,
        })
        .with(Potion {})
        .with(Consumable {
            use_verb: "quaff".into(),
        })
        .with(ProvidesHealing { heal_amount: 15 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Item {})
        .with(Name {
            name: "Magic Missile Scroll".into(),
        })
        .with(Position::new(x, y))
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 3,
        })
        .with(Consumable {
            use_verb: "read".into(),
        })
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Item {})
        .with(Name {
            name: "Fireball Scroll".into(),
        })
        .with(Position::new(x, y))
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Consumable {
            use_verb: "read".into(),
        })
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Item {})
        .with(Name {
            name: "Confusion Scroll".into(),
        })
        .with(Position::new(x, y))
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Consumable {
            use_verb: "read".into(),
        })
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Item {})
        .with(Name {
            name: "Dagger".to_string(),
        })
        .with(Position::new(x, y))
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Item {})
        .with(Name {
            name: "Shield".to_string(),
        })
        .with(Position::new(x, y))
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Item {})
        .with(Name {
            name: "Longsword".to_string(),
        })
        .with(Position::new(x, y))
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Equippable {
            slot: EquipmentSlot::Melee,
        })
        .with(MeleePowerBonus { power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Item {})
        .with(Name {
            name: "Tower Shield".to_string(),
        })
        .with(Position::new(x, y))
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Equippable {
            slot: EquipmentSlot::Shield,
        })
        .with(DefenseBonus { defense: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn rations(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Item{})
        .with(Name{ name : "Rations".to_string() })
        .with(Position::new(x, y))
        .with(Renderable{
            glyph: rltk::to_cp437('%'),
            fg: RGB::named(rltk::GREEN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(ProvidesFood{})
        .with(Consumable{ use_verb: "eat".into() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_mapping_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position::new( x, y ))
        .with(Renderable{
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN3),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Name{ name : "Scroll of Magic Mapping".to_string() })
        .with(Item{})
        .with(MagicMapper{})
        .with(Consumable{ use_verb: "read".into() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Name{ name : "Bear Trap".to_string() })
        .with(Position::new(x, y ))
        .with(Renderable{
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2
        })
        .with(Hidden {} )
        .with(EntryTrigger {})
        .with(InflictsDamage { damage: 6 })
        .with(SingleActivation {})
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}