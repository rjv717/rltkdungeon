use specs::prelude::*;
use specs::saveload::{ConvertSaveload, Marker, SimpleMarker};
use specs::error::NoError;
use specs_derive::*;
use serde::{Serialize, Deserialize};

/// The register_components functio in this file is called once by the main function to tell
/// the ECS about all of the feature components that we use to modify entities (Player, Monsters, 
/// and possibly [future development] Items) at runtime.
/// 
/// This function is defined here so that we can more easily keep track of the components that we 
/// have placed in this module.
pub fn register_components(ecs: &mut World) {
    ecs.register::<EntityMoved>();
    ecs.register::<InBackpack>();
    ecs.register::<InCombat>();
    ecs.register::<SimpleMarker<SerializeMe>>();
    ecs.register::<SerializationHelper>();
    ecs.register::<SufferDamage>();
    ecs.register::<WantsToMelee>();
    ecs.register::<WantsToPickupItem>();
    ecs.register::<WantsToUseItem>();
    ecs.register::<WantsToDropItem>();
    ecs.register::<WantsToRemoveItem>();
}

/// The EntityMoved component is inserted on the player so that the trigger system can 
/// detect movement and act on it.
#[derive(Component, Debug, Serialize, Deserialize, Clone)]
pub struct EntityMoved {}

/// InBackpack is inserted on Items to tell the systems that this Item has been picked up 
/// and is being carried around by either the Player, or [future development] a Monster.
#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct InBackpack {
    pub owner : Entity
}

/// InCombat is used by the melee_combat_system to indicate which entities are currently 
/// fighting each other.
/// 
/// the initiative variable is a not yet developed system for determining who moves first. 
/// Currently damage is being added up as though everbody move at the same time.
#[derive(Component, PartialEq, ConvertSaveload, Debug, Clone)]
pub struct InCombat {
    pub initiative: i32,
}

// Special component that exists to help serialize and save the game data to disk.
#[derive(Component, Serialize, Deserialize, Clone)]
pub struct SerializationHelper {
    pub map : crate::map_builders::map::Map
}

/// The SerializeMe structure is used as a marker to indicate what things are to be recorded 
/// when we save the game to disk.
pub struct SerializeMe;

/// SuferDamage contains a vector of integer numbers to be added up and taken away from the
/// entity it is attached to when the damage_system runs.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct SufferDamage {
    pub amount : Vec<i32>
}

impl SufferDamage {
    /// We have implemented a new_damage routine to simplify and contain haow damage is added 
    /// to an entity for ease of management.
    pub fn new_damage(store: &mut WriteStorage<SufferDamage>, victim: Entity, amount: i32) {
        if let Some(suffering) = store.get_mut(victim) {
            suffering.amount.push(amount);
        } else {
            let dmg = SufferDamage { amount : vec![amount] };
            store.insert(victim, dmg).expect("Unable to insert damage");
        }
    }
}

/// WantsToMelee indicates that this Entity wants to attack the Entity recorded in the target 
/// variable within it.
#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct WantsToMelee {
    pub target : Entity
}

/// WantsToPickupItem tells the item_collection_system that Entity in collected_by is trying to 
/// pick up the Item in the item variable.
#[derive(Component, ConvertSaveload, Debug, Clone)]
pub struct WantsToPickupItem {
    pub collected_by : Entity,
    pub item : Entity
}

/// WantsToUseItem tells the item_use_system that the Entity that it is attached to is trying to 
/// use the Item recorded in item on the location recorded in target.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct WantsToUseItem {
    pub item: Entity,
    pub target : Option<rltk::Point>,
}

/// WantsToDropItem tells the item_drop_system that the entity that it is attached to wants to drop
/// the Item recorded in the item variable.
#[derive(Component, ConvertSaveload, Clone, Debug)]
pub struct WantsToDropItem {
    pub item: Entity,
}

/// WantsToRemoveItem tells the item_remove_system that the entity that it is attached to stop using 
/// an item that has been previously equipped. 
#[derive(Component, Debug, ConvertSaveload, Clone)]
pub struct WantsToRemoveItem {
    pub item : Entity
}

// Tests go after this point.

#[cfg(tests)]
mod tests {
    use components;

    #[test]
    fn position_create() {
        use components::Position;
        let here = Position::new(10, 10);

        assert_eq!(here.x, 10);
        assert_eq!(here.y, 10);
    }

    #[test]
    fn position_renderable() {
        use components::Position;
        let here = Position::new(10, 10);

        here.set_position(20, 20);

        assert_eq!(here.x, 20);
        assert_eq!(here.y, 20);
    }
}
