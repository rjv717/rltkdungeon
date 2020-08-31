use rltk::RandomNumberGenerator;
use std::collections::HashMap;

use components;

#[derive(Default)]
pub struct LookupTable {
    entries : HashMap<String, String>,
}

impl LookupTable {
    pub fn new() -> LookupTable {
        LookupTable{ entries: HashMap::new() }
    }

    pub fn add<S:ToString>(mut self, key : S, entry: S) -> LookupTable {
        self.entries.insert(key, entry);
        self
    }

    pub fn get<C>
    /*pub fn roll(&self, rng : &mut RandomNumberGenerator) -> String {
        if self.total_weight == 0 { return "None".to_string(); }
        let mut roll = rng.roll_dice(1, self.total_weight)-1;
        let mut index : usize = 0;

        while roll > 0 {
            if roll < self.entries[index].weight {
                return self.entries[index].name.clone();
            }

            roll -= self.entries[index].weight;
            index += 1;
        }

        "None".to_string()
    }*/
}