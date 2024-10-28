use std::collections::HashMap;
use rand::Rng;
use rand::seq::SliceRandom;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CityID(pub u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct EventID(u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CharacterID(pub u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Year(i32);

const NUM_LAYERS: usize = 5;
const NUM_CHARACTERS: usize = 3;

pub struct World {
    pub cities: HashMap<CityID, City>,
    // events: HashMap<EventID, Event>,
    pub characters: HashMap<CharacterID, Character>,
    city_id_counter: u32,
    event_id_counter: u32,
    character_id_counter: u32,
    
    pub layers: [Vec<CityID>; NUM_LAYERS]
}

impl World {
    pub fn new() -> Self {
        World {
            cities: HashMap::new(),
            characters: HashMap::new(),
            city_id_counter: 0,
            event_id_counter: 0,
            character_id_counter: 0,
            
            layers: [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()]
        }
    }
    
    pub fn generate_world() -> Self {
        let mut world = World::new();
       
        // add end city
        world.layers[NUM_LAYERS - 1] = vec![world.add_city(NUM_LAYERS - 1)];
        
        // add in between cities - we will work from end to beginning so that each city is guaranteed to be connected to at least one city from the next layer
        for layer in (1..=NUM_LAYERS - 2).rev() {
            let num_cities = rand::thread_rng().gen_range(2..=4);
            
            for _ in 1..=num_cities {
                // initialise new city
                let new_city = world.add_city(layer);
                world.layers[layer].push(new_city);
                
                // randomly connect to other cities in next layer
                let num_in_next_layer = world.layers[layer+1].len();
                let num_connections = rand::thread_rng().gen_range(1..=num_in_next_layer);
                let cities_in_next_layer = world.layers[layer+1].clone();
                let cities_to_connect = cities_in_next_layer.choose_multiple(&mut rand::thread_rng(), num_connections);
                
                for city_id in cities_to_connect {
                    world.connect_cities(&new_city, &city_id);
                }
            }  
        }
        
        // add start city
        
        let start_city = world.add_city(0);
        world.layers[0] = vec![start_city];
        
        for city_id in world.layers[1].clone() {
            world.connect_cities(&start_city, &city_id);
        }

        // add characters at their start positions (for now just the first city)
        for _ in (0..NUM_CHARACTERS) {
            let pns = Pronouns {
                nominative: "pn1".to_string(),
                accusative: "pn2".to_string(),
                dep_genitive: "pn3".to_string()
            };
            let new_character = world.add_character("test".to_string(), pns, world.layers[0][0]);
        }
        
        world
    }
    
    fn add_city(&mut self, layer: usize) -> CityID {
        let id = self.city_id_counter;
        self.city_id_counter += 1;
        
        let name = City::nameGen();
        
        let city = City::new(name);
        self.cities.insert(CityID(id), city);
        
        
        CityID(id)
    }
    
    fn connect_cities(&mut self, id1: &CityID, id2: &CityID) {
        let mut city1 = self.cities.get_mut(id1).unwrap();
        city1.neighbours.push(*id2);
        
        let mut city2 = self.cities.get_mut(id2).unwrap();
        city2.neighbours.push(*id1);
    }
    
    fn add_character(&mut self, name: String, pronouns: Pronouns, CityID(start_city_id): CityID) {
        if (self.city_id_counter <= start_city_id) {
            // error
        }

        let id = self.character_id_counter;
        self.character_id_counter += 1;
        let char = Character::new(name, pronouns);
        self.characters.insert(CharacterID(id), char);
        CharacterID(id);
    }
}

pub struct City {
    pub name: String,
    pub neighbours: Vec<CityID>,
    pub events: Vec<EventID>
}

const SOFTLETTERS: &'static [&'static str] = &["sh", "l", "m", "n", "r"];
const HARDLETTERS: &'static [&'static str] = &["p", "b", "t", "g"];
const VOWELS: &'static [&'static str] = &["a", "e", "i", "o", "oo", "ai"];

const SUFFIXES: &'static [&'static str] = &["ford", "ton", "don", "dale", "by"];


impl City {
    pub fn new(name: String) -> Self {
        City {
            name,
            neighbours: Vec::new(),
            events: Vec::new()
        }
    }
    
    pub fn nameGen() -> String {
        let mut first_syllable = "".to_string();
        
        first_syllable.push_str(HARDLETTERS.choose(&mut rand::thread_rng()).expect(""));
        first_syllable.push_str(VOWELS.choose(&mut rand::thread_rng()).expect(""));
        first_syllable.push_str(SOFTLETTERS.choose(&mut rand::thread_rng()).expect(""));
        first_syllable.push_str(SUFFIXES.choose(&mut rand::thread_rng()).expect(""));
        
        first_syllable
    }
}

#[derive(Debug)]
pub struct Character {
    // used in textgen
    pub name: String,
    pub pronouns: Pronouns,
    pub events: Vec<EventID>
}

#[derive(Debug)]
pub struct Pronouns {
    pub nominative: String, // she, him, they etc.
    pub accusative: String, // her, him, them etc.
    pub dep_genitive: String // her, his, their etc.
}

impl Character {
    pub fn new(name: String, pronouns: Pronouns) -> Self {
        Character {
            name: name,
            pronouns: pronouns,
            events: Vec::new()
        }
    }
}
