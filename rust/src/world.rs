use std::collections::HashMap;
use rand::Rng;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CityID(pub u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct EventID(u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CharacterID(u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Year(i32);

const NUM_LAYERS: i32 = 5;

pub struct World {
    pub cities: HashMap<CityID, City>,
    // events: HashMap<EventID, Event>,
    characters: HashMap<CharacterID, Character>,
    city_id_counter: u32,
    event_id_counter: u32,
    character_id_counter: u32,
    
    pub layers: [Vec<CityID>; NUM_LAYERS]
}

impl World {
    pub fn new() -> Self {
        World {
            cities: HashMap::new(),
            city_id_counter: 0,
            event_id_counter: 0,
            character_id_counter: 0,
            
            layers: [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()]
        }
    }
    
    pub fn generate_world() -> Self {
        let mut world = World::new();
       
        // add start and end cities
        world.layers[0] = vec![world.add_city(0)];
        world.layers[4] = vec![world.add_city(4)];
        
        // add in between cities
        for layer in (1..=3) {
            let num_cities = rand::thread_rng().gen_range(2..=4);
            for _ in (1..=num_cities) {
                let new_city = world.add_city(layer);
                world.layers[layer].push(new_city);
                
                for city_id in world.layers[layer-1].clone() {
                    let mut prev_city = world.cities.get_mut(&city_id).unwrap();
                    prev_city.neighbours.push(new_city);
                    
                    let mut new_city = world.cities.get_mut(&new_city).unwrap();
                    new_city.neighbours.push(city_id);
                }
            }
        }
        
        world
    }
    
    fn add_city(&mut self, layer: usize) -> CityID {
        let id = self.city_id_counter;
        self.city_id_counter += 1;
        
        let city = City::new();
        self.cities.insert(CityID(id), city);
        CityID(id)
    }

    fn add_character(&mut self, start_city: CityID) {
        if (self.city_id_counter <= start_city) {
            // error
        }

        let id = self.character_id_counter;
        self.character_id_counter += 1;
        let char = Character::new()
        self.characters.insert(CharacterID(id), char);
        CharacterID(id);
    }
}

pub struct City {
    pub name: String,
    pub neighbours: Vec<CityID>,
    pub events: Vec<EventID>
}

impl City {
    pub fn new() -> Self {
        City {
            name: "".to_string(),
            neighbours: Vec::new(),
            events: Vec::new()
        }
    }
}

pub struct Character {
    // used in textgen
    pub name: String,
    pub pronouns: [String, 3], // 1: nominative (ex. she, him, they) 2: accusative (ex: her, him, them) 3: dependent genitive (ex: her, his, their)
    pub events: Vec<EventID>
}

impl Character {
    pub fn new() -> Self {
        Person {
            name: "".to_string(),
            events: Vec::new()
        }
    }
}
