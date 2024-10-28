use rand::Rng;
use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CityID(pub u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct EventID(pub u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CharacterID(pub u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Year(i32);

const NUM_LAYERS: usize = 5;
const NUM_CHARACTERS: usize = 3;

// -- World and world gen --

pub struct World {
    pub cities: HashMap<CityID, City>,
    // events: HashMap<EventID, Event>,
    pub characters: HashMap<CharacterID, Character>,
    pub events: HashMap<EventID, Event>,
    city_id_counter: u32,
    event_id_counter: u32,
    character_id_counter: u32,

    pub layers: [Vec<CityID>; NUM_LAYERS],
}

impl World {
    pub fn new() -> Self {
        World {
            cities: HashMap::new(),
            characters: HashMap::new(),
            events: HashMap::new(),
            city_id_counter: 0,
            event_id_counter: 0,
            character_id_counter: 0,

            layers: [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        }
    }

    pub fn generate_world() -> Self {
        let mut world = World::new();

        // add start and end cities
        world.layers[0] = vec![world.add_city(0)];
        world.layers[NUM_LAYERS - 1] = vec![world.add_city(NUM_LAYERS - 1)];

        // add in between cities
        for layer in (1..=NUM_LAYERS - 2) {
            let num_cities = rand::thread_rng().gen_range(2..=4);
            for _ in (1..=num_cities) {
                let new_city = world.add_city(layer);
                world.layers[layer].push(new_city);

                for city_id in world.layers[layer - 1].clone() {
                    let mut prev_city = world.cities.get_mut(&city_id).unwrap();
                    prev_city.neighbours.push(new_city);

                    let mut new_city = world.cities.get_mut(&new_city).unwrap();
                    new_city.neighbours.push(city_id);
                }
            }
        }

        // add characters at their start positions (for now just the first city)
        for _ in (0..NUM_CHARACTERS) {
            let pns = Pronouns {
                nominative: "pn1".to_string(),
                accusative: "pn2".to_string(),
                dep_genitive: "pn3".to_string(),
            };
            world.add_character("test".to_string(), pns, world.layers[0][0]);
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

    fn add_character(
        &mut self,
        name: String,
        pronouns: Pronouns,
        CityID(start_city_id): CityID,
    ) -> CharacterID {
        if (self.city_id_counter <= start_city_id) {
            panic!("Invalid start city ID: {}", start_city_id); // invalid id
        }

        let id = self.character_id_counter;
        self.character_id_counter += 1;
        let char = Character::new(name, pronouns);
        self.characters.insert(CharacterID(id), char);
        CharacterID(id)
    }
}

// -- City class --

pub struct City {
    pub name: String,
    pub neighbours: Vec<CityID>,
    pub events: Vec<EventID>,
}

impl City {
    pub fn new() -> Self {
        City {
            name: "".to_string(),
            neighbours: Vec::new(),
            events: Vec::new(),
        }
    }
}

// -- Character class and associated classes

#[derive(Debug)]
pub struct Character {
    // used in textgen
    pub name: String,
    pub pronouns: Pronouns,
    pub events: Vec<EventID>,
}

#[derive(Debug)]
pub struct Pronouns {
    // probably gonna be needed for text gen
    pub nominative: String, // she, him, they etc.
    pub accusative: String, // her, him, them etc.
    pub dep_genitive: String, // her, his, their etc.
                            // could add more if needed
}

impl Character {
    pub fn new(name: String, pronouns: Pronouns) -> Self {
        Character {
            name: name,
            pronouns: pronouns,
            events: Vec::new(),
        }
    }
}

// -- Events --

// event types
#[derive(Debug)]
pub enum EventType {
    EventDeath,        // an event representing the death of a character.
    EventEncounter, // an event representing a fleeting encounter between two people. An alive character could encounter a dead character.
    EventMoveTogether, // an event representing two characters moving together for a while.
}

// An event that has a start time and maybe an end time.
#[derive(Debug)]
pub struct Event {
    characters: Vec<CharacterID>, // characters in the event
    start_time: usize,
    end_time: Option<usize>,
    event_type: EventType,
    events_happening_during: Vec<EventID>,
}

// Helper function to check if two event times overlap
pub fn durations_overlap(
    start1: usize,
    start2: usize,
    end1: Option<usize>,
    end2: Option<usize>,
) -> bool {
    if (start1 == start2) {
        true;
    }
    match (end1, end2) {
        (None, None) => false,
        (Some(e1), None) => start2 > start1 && start2 < e1,
        (None, Some(e2)) => start1 > start2 && start1 < e2,
        (Some(e1), Some(e2)) => !(start1 > e2 || e1 < start2),
    }
}

impl Event {
    pub fn new(
        characters: Vec<CharacterID>,
        start_time: usize,
        end_time: Option<usize>,
        event_type: EventType,
    ) -> Self {
        Event {
            characters,
            start_time,
            end_time,
            event_type,
            events_happening_during: Vec::new(),
        }
    }

    // TODO: to be tested when events are added into world gen
    pub fn add_event_during(&mut self, world: World, event_id: EventID) {
        match world.events.get(&event_id) {
            Some(&ref event) => {
                let valid: bool = durations_overlap(
                    self.start_time,
                    event.start_time,
                    self.end_time,
                    event.end_time,
                );
                if valid {
                    self.events_happening_during.push(event_id);
                }
            }
            None => match event_id {
                EventID(id) => panic!("No event found associated with eventID {}", id),
            },
        }
    }
}
