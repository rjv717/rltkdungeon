//! All of the "systems" that are run on every (nearly?) game tick are placed here to 
//! contain them all in one place.
 
/// damage_system collects damage amounts done and applies them every game tick. 
pub mod damage_system;

/// hunger_system ticks down hunger counter on every turn.
/// 
/// TODO: Setup helper functions to add in (possibly like the damage_system) differing 
/// amounts of food value that different food could have.
pub mod hunger_system;

/// inventory_systems contian all of the "systems" involved in controlling the Items in
/// "inventory", that the Player or [future development] Monsters could pick up nd try 
/// to use.
pub mod inventory_systems;

/// The map_indexing_system keeps the locations of objects within the map up to date.
pub mod map_indexing_system;

/// The melee_combat_system tracks combat actions and applies all of the actions as needed.
pub mod melee_combat_system;

/// The monster_ai_system makes the Monsters "think".  Currently they don't a lot of "thinking"
/// aside from noticing the player and chasing to attack.
pub mod monster_ai_system;

/// The particle_system controls the special effects displayed on the map when certain 
/// actions are taken.
pub mod particle_system;

/// The saveload_system records game saves to disk and loads them back in.
pub mod saveload_system;

/// The traps use the trigger_system tell them when to go off.
pub mod trigger_system;

/// All of the routines to display the User information systems are contained here.
pub mod ui_system;

/// The visibility_system controls and updates who can see what.
pub mod visiblity_system;
