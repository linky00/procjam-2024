use std::collections::HashMap;
use rand::Rng;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
struct CityID(u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
struct EventID(u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
struct CharacterID(u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
struct Year(i32);

struct World {
    cities: HashMap<CityID, City>,
    // events: HashMap<EventID, Event>,
    // characters: HashMap<CharacterID, Character>,
    city_id_counter: u32,
    event_id_counter: u32,
    character_id_counter: u32,
    
    layers: [Vec<CityID>; 5]
}

impl World {
    fn new() -> Self {
        World {
            cities: HashMap::new(),
            city_id_counter: 0,
            event_id_counter: 0,
            character_id_counter: 0,
            
            layers: [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()]
        }
    }
    
    fn generate_world() -> Self {
        let mut world = World::new();
       
        
        world.layers[0] = vec![world.add_city(0)];
        world.layers[4] = vec![world.add_city(4)];
        
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
}

struct City {
    name: String,
    neighbours: Vec<CityID>,
    events: Vec<EventID>
}

impl City {
    fn new() -> Self {
        City {
            name: "".to_string(),
            neighbours: Vec::new(),
            events: Vec::new()
        }
    }
}


fn main() {
    let mut world = World::generate_world();
    println!("{:?}",world.layers[0]);
    println!("{:?}",world.layers[1]);
    println!("{:?}",world.layers[2]);
    println!("{:?}",world.layers[3]);
    println!("{:?}",world.layers[4]);
    
    println!("{:?}", world.cities.get(&CityID(3)).unwrap().neighbours);
}
