use rltk::{FontCharType, Point, RGB};
use specs::*;
use specs::saveload::{ConvertSaveload, Marker};
use specs::error::NoError;
use specs_derive::*;
use serde::{Serialize, Deserialize};

use crate::map_builders::map::Map;
use super::{item::Item, item::Potion, monster::Monster, player::Player};

trait Build<T> {
    fn create (creator: String) -> Option<T>;
}

/// Like in the Components module we have included this function register_spawns to contain 
/// all of the component registrations for our spawnables.  Monsters, Player, and Items use
/// these components in their initial creation.
pub fn register_spawns(ecs: &mut World) {
    ecs.register::<Actor>();
    ecs.register::<AreaOfEffect>();
    ecs.register::<BlocksTile>();
    ecs.register::<CombatStats>();
    ecs.register::<Confusion>();
    ecs.register::<Consumable>();
    ecs.register::<DefenseBonus>();
    ecs.register::<Equippable>();
    ecs.register::<Equipped>();
    ecs.register::<EntryTrigger>();
    ecs.register::<Hidden>();
    ecs.register::<HungerClock>();
    ecs.register::<InflictsDamage>();
    ecs.register::<Item>();
    ecs.register::<MagicMapper>();
    ecs.register::<MeleePowerBonus>();
    ecs.register::<Monster>();
    ecs.register::<Name>();
    ecs.register::<Player>();
    ecs.register::<Position>();
    ecs.register::<Potion>();
    ecs.register::<ProvidesFood>();
    ecs.register::<ProvidesHealing>();
    ecs.register::<Ranged>();
    ecs.register::<Renderable>();
    ecs.register::<SingleActivation>();
    ecs.register::<Viewshed>();
}

/// Come to think of it, I don't have a clue why this exists.  It is not currently being 
/// used anywhere.
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct Actor {}
#[allow(unused_variables)]
impl Build<Actor> for Actor {
    fn create(key: String) -> Option<Actor> {
        Some(Actor {})
    } 
}

/// The Fireball Scroll effects an area with a radius of 3. This Component tells the 
/// item_use_system that it effects an area rather than a single target.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct AreaOfEffect {
    pub radius : i32
}
impl Build<AreaOfEffect> for AreaOfEffect {
    fn create(key:String) -> Option<AreaOfEffect> {
        if key.is_empty() {return None;}
        let mut radius = 3;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "radius" => {radius = entry[1].parse().unwrap_or(3);},
                _ => {} 
            }
        }
        Some(AreaOfEffect{ radius: radius })
    }
}

/// This component tells the map_indexing_system that this Entity blocks the tile so nobody 
/// else can pass.
#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct BlocksTile {}
#[allow(unused_variables)]
impl Build<BlocksTile> for BlocksTile {
    fn create (key:String) -> Option<BlocksTile> {
        Some(BlocksTile {})
    }
}

/// CombatStats contains the base combat abilities of the Entity it is attached to.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct CombatStats {
    pub hp: i32,
    pub hp_max : i32,
    pub attack: i32,
    pub defense: i32,
    pub is_dead: bool
}
impl Build<CombatStats> for CombatStats {
    fn create(key: String) -> Option<CombatStats> {
        if key.is_empty() {return None;}
        let mut hp = 15;
        let mut attack = 4;
        let mut defense = 2;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "hp" => {hp = entry[1].parse().unwrap_or(15);},
                "attack" => {attack = entry[1].parse().unwrap_or(4)},
                "defense" => {defense = entry[1].parse().unwrap_or(2)},
                _ => {}
            }
        }
        Some( CombatStats { hp, hp_max: hp, attack, defense, is_dead: false  })
    }
}
impl CombatStats {
    /// We can place checks and modifiers to how much damage is incurred in this 
    /// damage function.
    pub fn damage(&mut self, amount: i32) {
        self.hp -= amount;
        if self.hp < 1 {
            self.is_dead = true;
        }
    }

    /// We can place check and modifiers to healing ablity in this function.
    pub fn heal(&mut self, amount: i32) {
        self.hp += amount;
        if self.hp > self.hp_max { 
            self.hp = self.hp_max;
        }
    }
}

/// Is the Entity under the effect of Confuision? How many turns are left on the Confusion
/// effect? This comonent is the control for answering those questions.
/// 
/// TODO: This should probably be moved to the top-level Components module.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Confusion {
    pub turns: i32
}
impl Build<Confusion> for Confusion {
    fn create(key:String) -> Option<Confusion> {
        if key.is_empty() {return None;}
        let mut turns = 4;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "turns" => {turns = entry[1].parse().unwrap_or(4);},
                _ => {}
            }
        }
        Some(Confusion {turns})
    }
}

/// The Entity that this component is connected to will be use up and destroyed when the 
/// Item is used.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Consumable {
    pub use_verb: String,
}
impl Build<Consumable> for Consumable {
    fn create(key:String) -> Option<Consumable> {
        if key.is_empty() {return None;}
        let mut use_verb = "use".into();
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "use_verb" => { use_verb = entry[1].parse().unwrap_or("use".into())},
                _ => {}
            }
        }
        Some(Consumable { use_verb })
    }
}

/// This component indicates that this Entity is has a bonus to its defense, due to equiping 
/// some Item like a shield.
/// 
/// TODO: It might be good to intigrate this effect into CombatStats and reduce the number of 
/// components.
#[derive(Component, ConvertSaveload, Clone)]
pub struct DefenseBonus {
    pub defense : i32
}
impl Build<DefenseBonus> for DefenseBonus {
    fn create(key: String) -> Option<DefenseBonus> {
        if key.is_empty() { return None; }
        let mut defense = 2;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0].into() {
                "defense" => {defense = entry[1].parse().unwrap_or(2);},
                _ => {}
            }
        }
        Some( DefenseBonus { defense } )
    }
}

/// Enumeration tag used by Equippable to indicate how this Item is used, when it is Equipped.
#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum EquipmentSlot { Melee, Shield, None }

/// This Component tells the code that this Item can be equipped and what type of equipment it 
/// is.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct Equippable {
    pub slot : EquipmentSlot
}
impl Build<Equippable> for Equippable {
    fn create(key:String) -> Option<Equippable> {
        if key.is_empty() { return None; }
        let mut slot = EquipmentSlot::None;
        for item in key.split_ascii_whitespace() {
            match item {
                "melee" => {slot = EquipmentSlot::Melee;},
                "shield" => {slot = EquipmentSlot::Shield;},
                _ => { return None; }
            }
        }
        Some( Equippable { slot })
    }
}

/// This Component tell the code that the Item it is inserted on has been equipped and what role
/// it play as equipment.
/// 
/// TODO: This should perhaps be refactored into the top-level Component module.
#[derive(Component, ConvertSaveload, Clone)]
pub struct Equipped {
    pub owner : Entity,
    pub slot : EquipmentSlot
}

/// This component tells the code that the generated Item has a trigger and the trigger_system 
/// should act on it. 
/// 
/// TODO: Currently the only Trigger used by the trigger_system is movement in the Items range. 
/// We could add other triggers for different effects later on.
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntryTrigger {}
#[allow(unused_variables)]
impl Build<EntryTrigger> for EntryTrigger {
    fn create(key:String) -> Option<EntryTrigger> {
        Some( EntryTrigger {} )
    }
}

/// GeneralStats is not yet implemented. I placed it in here intending to implement modifiers 
/// based on randomized Stats included here, but haven't gotten to that yet.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct GeneralStats {
    pub strength: u32,
    pub dexterity: u32,
    pub constution: u32,
    pub intelligence: u32
}

/// The Hidden component is a simple tag to indicate that the Item it is attached to should not 
/// be displayed, ie. hidden.  
/// 
/// Currently only the Trap uses this tag.
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct Hidden {}
#[allow(unused_variables)]
impl Build<Hidden> for Hidden {
    fn create(key:String) -> Option<Hidden> {
        Some( Hidden {} )
    }
}

/// HungerState is used by the HungerClock to indicate and display how hungry the player is.
#[derive(PartialEq, Serialize, Deserialize, Clone, Debug)]
pub enum HungerState {
    WellFed, 
    Normal,
    Hungry,
    Starving
}

/// The HUngerClock determines when your HungerState changes.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct HungerClock {
    pub state: HungerState,
    pub duration: i32,
}
impl Build<HungerClock> for HungerClock {
    fn create(key: String) -> Option<HungerClock> {
        if key.is_empty() {return None;}
        let mut duration = 400;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "duration" => { duration = entry[1].parse().unwrap_or(400); },
                _ => {}
            }
        }
        Some( HungerClock { state: HungerState::WellFed, duration } )
    }
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct InflictsDamage {
    pub damage: i32,
}
impl Build<InflictsDamage> for InflictsDamage {
    fn create (key: String) -> Option<InflictsDamage> {
        if key.is_empty() {return None;}
        let mut damage=4;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "damage" => { damage = entry[1].parse().unwrap_or(4); },
                _ => {}
            }
        }
        Some( InflictsDamage { damage } )
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct MagicMapper {  }
#[allow(unused_variables)]
impl Build<MagicMapper> for MagicMapper {
    fn create(key: String) -> Option<MagicMapper> {
        Some ( MagicMapper { } )
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct MeleePowerBonus {
    pub power : i32
}
impl Build<MeleePowerBonus> for MeleePowerBonus {
    fn create(key: String) -> Option<MeleePowerBonus> {
        if key.is_empty() { return None; }
        let mut power=2;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "power" => { power = entry[1].parse().unwrap_or(2); }
                _ => {}
            }
        }
        Some( MeleePowerBonus { power } )
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Name{
    pub name: String,
}
impl Build<Name> for Name {
    fn create(key: String) -> Option<Name> {
        if key.is_empty() { return None; }
        let mut name: String = "Unknown".into();
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "name" => { name = String::from(entry[1]).replace("_", " ");},
                _ => { }
            }
        }
        Some ( Name { name } )
    }
}

#[derive(Component, ConvertSaveload, Clone, Copy, Debug)]
pub struct Position {
    x: i32,
    y: i32,
}
impl Build<Position> for Position {
    fn create(key: String) -> Option<Position> {
        if key.is_empty() { return None; }
        let mut x = 0;
        let mut y = 0;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "x" => { x = entry[1].parse().unwrap_or(0); },
                "y" => { y = entry[1].parse().unwrap_or(0); },
                _ => {}
            }
        }
        Some (Position { x, y } )
    }
}
impl Position {
    pub fn new(x: i32, y: i32) -> Position {
        Position { x, y }
    }

    #[allow(dead_code)]
    pub fn set_x(&mut self, x: i32) {
        self.x = x;
    }

    pub fn get_x(self) -> i32 {
        self.x
    }

    #[allow(dead_code)]
    pub fn set_y(&mut self, y: i32) {
        self.y = y;
    }

    pub fn get_y(self) -> i32 {
        self.y
    }

    pub fn try_move(&mut self, map: &Map, delta_x: i32, delta_y:i32) -> bool {
        let x = self.x + delta_x;
        let y = self.y + delta_y;

        if x < 1 || x > map.width-1 || y < 1 || y > map.width-1 || map.is_blocked(x, y) {
            return false;
        }
        
        self.x = x;
        self.y = y;

        true
    }
}

#[derive(Component, Serialize, Deserialize, Clone, Debug)]
pub struct ProvidesFood {}
#[allow(unused_variables)]
impl Build<ProvidesFood> for ProvidesFood {
    fn create(key: String) -> Option<ProvidesFood> {
        Some( ProvidesFood { } )
    }
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct ProvidesHealing {
    pub heal_amount : i32
}
impl Build<ProvidesHealing> for ProvidesHealing {
    fn create(key: String) -> Option<ProvidesHealing> {
        if key.is_empty() { return None; } 
        let mut heal_amount = 15;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "heal_amount" => { heal_amount = entry[1].parse().unwrap_or(15); },
                _ => {}
            }
        }
        Some ( ProvidesHealing { heal_amount } )
    }
}

#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct Ranged {
    pub range: i32,
}
impl Build<Ranged> for Ranged {
    fn create(key: String) -> Option<Ranged> {
        if key.is_empty() { return None; } 
        let mut range=8;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "range" => { range = entry[1].parse().unwrap_or(8); },
                _ => {}
            }
        }
        Some ( Ranged { range } )
    }
}

#[derive(Component, ConvertSaveload, Clone, Copy)]
pub struct Renderable {
    pub glyph: FontCharType,
    pub fg: RGB,
    pub bg: RGB,
    pub render_order: i32,
}
impl Build<Renderable> for Renderable {
    fn create(key: String) -> Option<Renderable> {
        if key.is_empty() { return None; } 
        let mut glyph = 0x007E;
        let mut fg = RGB::named(rltk::GREEN);
        let mut bg = RGB::named(rltk::BLACK);
        let mut render_order = 3;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "glyph" => { glyph = entry[1].encode_utf16().nth(0).unwrap_or(0x007E); }, // 0x007E = '~' in utf-16
                "fg" => { fg = select_RGB(entry[1].parse().unwrap_or("GREEN".into())); },
                "bg" => { bg = select_RGB(entry[1].parse().unwrap_or("BLACK".into())); },
                "render_odrer" => { render_order = entry[1].parse().unwrap_or(3); },
                _ => {}
            }
        }
        Some ( Renderable { glyph: glyph, fg, bg, render_order } )
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct SingleActivation {}
#[allow(unused_variables)]
impl Build<SingleActivation> for SingleActivation {
    fn create(key: String) -> Option<SingleActivation> {
        Some ( SingleActivation {} )
    }
}

#[derive(Component, ConvertSaveload, Clone)]
pub struct Viewshed {
    pub visible_tiles: Vec<Point>,
    pub range: i32,
    pub dirty: bool,
}
impl Build<Viewshed> for Viewshed {
    fn create(key: String) -> Option<Viewshed> {
        if key.is_empty() { return None; }
        let mut range = 8;
        for item in key.split_ascii_whitespace() {
            let entry:Vec<&str> = item.split(":").collect();
            match entry[0] {
                "range" => { range = entry[1].parse().unwrap_or(8); },
                _ => {}
            }
        }
        Some ( Viewshed { visible_tiles: Vec::new(), range, dirty: true } )
    }
}

#[allow(non_snake_case)]
fn select_RGB(key:String) -> RGB {
    match &key[0..key.len()] {
        "WHITE" => RGB::named(rltk::WHITE),
        "BLACK" => RGB::named(rltk::BLACK),
        "GREEN" => RGB::named(rltk::GREEN),
        "CYAN" => RGB::named(rltk::CYAN),
        "CYAN3" => RGB::named(rltk::CYAN3),
        "ORANGE" => RGB::named(rltk::ORANGE),
        "PINK" => RGB::named(rltk::PINK),
        "YELLOW" => RGB::named(rltk::YELLOW),
        "RED" => RGB::named(rltk::RED),
        "MAGENTA" => RGB::named(rltk::MAGENTA),
        _ => RGB::named(rltk::GRAY)
    }
}