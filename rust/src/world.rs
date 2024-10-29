use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CityID(pub u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct EventID(pub u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CharacterID(pub u32);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Year(i32);

// -- Constants --

const MAX_TIME: usize = 5;
const NUM_LAYERS: usize = 5;
const NUM_CHARACTERS: usize = 3;
const NUM_EVENTS: usize = 4;
const LIST_EVENTS: [EventType; NUM_EVENTS] = [
    EventType::EventMove,
    EventType::EventDeath,
    EventType::EventEncounter,
    EventType::EventIdle,
];

// -- World and world gen --

pub struct World {
    pub cities: HashMap<CityID, City>,
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
                let num_in_next_layer = world.layers[layer + 1].len();
                let num_connections = rand::thread_rng().gen_range(1..=num_in_next_layer);
                let cities_in_next_layer = world.layers[layer + 1].clone();
                let cities_to_connect =
                    cities_in_next_layer.choose_multiple(&mut rand::thread_rng(), num_connections);

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

        // add characters
        for _ in (0..NUM_CHARACTERS) {
            let pns = Pronouns {
                nominative: "pn1".to_string(),
                accusative: "pn2".to_string(),
                dep_genitive: "pn3".to_string(),
            };
            world.add_character("test".to_string(), pns);
        }

        world
    }

    fn add_city(&mut self, layer: usize) -> CityID {
        let id = self.city_id_counter;
        self.city_id_counter += 1;

        let name = City::name_gen();

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

    fn add_character(&mut self, name: String, pronouns: Pronouns) -> CharacterID {
        let id = self.character_id_counter;
        self.character_id_counter += 1;
        let char = Character::new(name, pronouns);
        self.characters.insert(CharacterID(id), char);
        CharacterID(id)
    }

    fn add_event(
        &mut self,
        characters: Vec<CharacterID>,
        start_time: usize,
        end_time: Option<usize>,
        event_type: EventType,
        event_place: CityID,
    ) -> EventID {
        let event_id = EventID(self.event_id_counter);
        self.event_id_counter += 1;
        let event = Event::new(characters, start_time, end_time, event_type);

        for char_id in event.characters.iter() {
            // add event to related characters' list of events
            let character = self.characters.get_mut(char_id).unwrap();
            character.events.push(event_id);

            // add event to city events
            let city = self.cities.get_mut(&event_place).unwrap();
            city.events.push(event_id);
        }

        // add event to event map
        self.events.insert(event_id, event);

        event_id
    }

    // generates events and places them in the event lists of cities and characters.
    // run generate_world before running this or perish in the doomed narrative that you've created
    pub fn generate_events(&mut self) {
        struct CharacterState {
            character: CharacterID,
            city: CityID,
            event_probability_map: WeightedIndex<usize>,
        }

        // set up initial states for each character
        let mut states = Vec::new(); // in order of character id
        for char_id in 0..NUM_CHARACTERS {
            let starting_weights: [usize; NUM_EVENTS] = [
                NUM_CHARACTERS / 2, // EventMove (this is a heuristic, we could change it)
                0,                  // EventDeath
                NUM_CHARACTERS / 2, // EventEncounter (keep this proportional to the number of other characters in the same city)
                0,                  // EventIdle
            ];
            let event_probability_map = WeightedIndex::new(&starting_weights).unwrap();
            println!(
                "starting event probability map: {:?}",
                event_probability_map
            );
            states.push(CharacterState {
                character: CharacterID(char_id.try_into().unwrap()),
                city: self.layers[0][0], // start city
                event_probability_map,
            });
        }

        // helper sub function to get characters in a city
        fn get_characters_in_city(city: CityID, states: &Vec<CharacterState>) -> Vec<CharacterID> {
            let mut chars_in_city = Vec::new();
            for state in states.iter() {
                if state.city == city {
                    chars_in_city.push(state.character);
                }
            }
            return chars_in_city;
        }

        // start running history
        let mut rng = thread_rng();
        let mut time = 0;
        while time < MAX_TIME {
            // recalculate city populations to use later in the loop if required
            let city_populations: Vec<Vec<CharacterID>> = self
                .cities
                .keys()
                .into_iter()
                .map(|&cityid| get_characters_in_city(cityid, &states))
                .collect();

            // determine next events for each character
            for state in states.iter_mut() {
                let next_event = &LIST_EVENTS[state.event_probability_map.sample(&mut rng)];

                // carry out event
                match next_event {
                    EventType::EventIdle => (), // do nothing (event idle is not logged)
                    EventType::EventMove => {
                        // determine city to move to
                        let curr_city = self.cities.get(&state.city).unwrap();
                        let &next_city = curr_city.neighbours.choose(&mut rng).unwrap();

                        // change character city to next city
                        state.city = next_city;

                        // recalculate probability map
                        let next_city_id_num: usize = match next_city {
                            CityID(id) => id.try_into().unwrap(),
                        };
                        let population = city_populations[next_city_id_num].len();
                        println!("Population of next city: {:?}", population);
                        // if the character moves to the last city, set probability of moving again to zero.
                        // otherwise, the probability is proportional to half the population of the city.
                        let last_city_id: usize = (self.city_id_counter - 1).try_into().unwrap();
                        let new_prob = if next_city_id_num == last_city_id {
                            0
                        } else {
                            population / 2
                        };
                        let weights_change_result = state
                            .event_probability_map
                            .update_weights(&[(0, &(population / 2)), (2, &new_prob)]);
                        println!("weights change result: {:?}", weights_change_result);

                        // add event to character's events
                        self.add_event(
                            vec![state.character],
                            time,
                            None,
                            EventType::EventMove,
                            next_city,
                        );

                        // print event
                        println!(
                            "Character with ID {:?} moved to City with ID {:?}",
                            state.character, next_city
                        );
                    }

                    EventType::EventDeath => {
                        // set all probabilities to zero except EventIdle
                        state.event_probability_map = WeightedIndex::new(&[0, 0, 0, 1]).unwrap();

                        // add death event
                        self.add_event(
                            vec![state.character],
                            time,
                            None,
                            EventType::EventMove,
                            state.city,
                        );

                        // print event
                        println!(
                            "Character with ID {:?} died in City with ID {:?}",
                            state.character, state.city
                        );
                    }

                    EventType::EventEncounter => {
                        // no change in probabilities

                        // pick random person from city to encounter
                        let city_id: usize = match state.city {
                            CityID(id) => id.try_into().unwrap(),
                        };
                        let &encountered = city_populations[city_id].choose(&mut rng).unwrap();

                        // add encounter event
                        self.add_event(
                            vec![state.character, encountered],
                            time,
                            None,
                            EventType::EventEncounter,
                            state.city,
                        );

                        // print event
                        println!(
                            "Character with ID {:?} encountered Character with ID {:?} in City with ID {:?}",
                            state.character, encountered, state.city
                        );
                    }
                }
            }

            time += 1;
        }
    }
}

// -- City class --

pub struct City {
    pub name: String,
    pub neighbours: Vec<CityID>,
    pub events: Vec<EventID>,
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
            events: Vec::new(),
        }
    }
    pub fn name_gen() -> String {
        let mut first_syllable = "".to_string();

        first_syllable.push_str(HARDLETTERS.choose(&mut rand::thread_rng()).expect(""));
        first_syllable.push_str(VOWELS.choose(&mut rand::thread_rng()).expect(""));
        first_syllable.push_str(SOFTLETTERS.choose(&mut rand::thread_rng()).expect(""));
        first_syllable.push_str(SUFFIXES.choose(&mut rand::thread_rng()).expect(""));

        first_syllable
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

// event types (the float is used for probability)
#[derive(Debug)]
pub enum EventType {
    EventMove,      // an event representin moving from one city to another
    EventDeath,     // an event representing the death of a character.
    EventEncounter, // an event representing a fleeting encounter between two people. An alive character could encounter a dead character.
    EventIdle,
    // EventMoveTogether, // an event representing two characters moving together for a while.
    // add more!
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
    if start1 == start2 {
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
