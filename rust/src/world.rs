use rand::distributions::WeightedIndex;
use rand::prelude::Distribution;
use rand::rngs::ThreadRng;
use rand::seq::{IteratorRandom, SliceRandom};
use rand::{thread_rng, Rng};
use std::cmp;
use std::collections::HashMap;

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CityID(pub usize);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct EventID(pub usize);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct CharacterID(pub usize);

#[derive(Eq, Hash, PartialEq, Copy, Clone, Debug)]
pub struct Year(isize);

// -- Constants --

const MAX_TIME: usize = 9;
const NUM_LAYERS: usize = 5;
const MIN_CITIES_IN_LAYER: usize = 1;
const MAX_CITIES_IN_LAYER: usize = 2;
const NUM_CHARACTERS: usize = 6;
const NUM_EVENTS: usize = 4; // number of event types
const CALAMITY_FREQ: usize = 1; // the frequency with which the calamity advances to the next layer
const CALAMITY_DEADLINESS: usize = 2; // calamity's kill probability increases with respect to this every time step
const LIST_EVENTS: [EventType; NUM_EVENTS] = [
    EventType::EventMove,
    EventType::EventDeath,
    EventType::EventEncounter,
    EventType::EventIdle,
];

// -- World and world gen --

// the state of a character at some given time
pub struct CharacterState {
    character: CharacterID,
    city: CityID,
    event_probability_map: WeightedIndex<usize>,
    dead: bool,
}

// the state of the calamity
pub struct CalamityState {
    pub city_states: HashMap<CityID, usize>, // how long each city has been "in calamity"
    pub freq: usize, // the frequency with which the calamity moves (once every freq time units)
    pub calamity_layer_i: Option<usize>, // which layer the calamity is currently at
}

impl CalamityState {
    pub fn new(cities: Vec<&CityID>) -> Self {
        let mut state = CalamityState {
            city_states: HashMap::new(),
            freq: CALAMITY_FREQ,
            calamity_layer_i: None,
        };

        for &city in cities {
            state.city_states.insert(city, 0);
        }

        state
    }

    pub fn calamity_step(
        &mut self,
        time: usize,
        layers: &Vec<Vec<CityID>>,
        character_states: &mut Vec<CharacterState>,
        city_populations: &HashMap<CityID, Vec<CharacterID>>,
    ) {
        if time % self.freq == 0 && time != 0 {
            // increase layer of calamity, if calamity isnt present yet, put it on layer 0
            match self.calamity_layer_i {
                Some(lyr) => {
                    if lyr < NUM_LAYERS - 1 {
                        self.calamity_layer_i = Some(lyr + 1);
                    }
                }
                None => {
                    self.calamity_layer_i = Some(0);
                }
            }
        }

        // increase calamity time of each city in calamity
        match self.calamity_layer_i {
            Some(calamity_layer_i_num) => {
                for layer_i in 0..=calamity_layer_i_num {
                    let layer = &layers[layer_i];
                    for &city_id in layer {
                        let prev_city_state = self.city_states.get(&city_id).unwrap();
                        self.city_states.insert(city_id, prev_city_state + 1);
                    }
                }
            }
            None => (),
        }

        // update death probabilities of each character in calamity
        match self.calamity_layer_i {
            Some(calamity_layer_i_num) => {
                for layer_i in 0..=calamity_layer_i_num {
                    let layer = &layers[layer_i];
                    for &city_id in layer {
                        let city_population = city_populations.get(&city_id).unwrap();
                        let &city_state = self.city_states.get(&city_id).unwrap();
                        for &CharacterID(id) in city_population {
                            let char_state = &character_states[id];
                            // dont update states if character is already dead
                            if !char_state.dead {
                                let last_city_id: usize = 0;
                                let curr_city_id_num = match city_id {
                                    CityID(id) => id,
                                };
                                let new_move_prob = if curr_city_id_num == last_city_id {
                                    0
                                } else {
                                    city_population.len() / 4 + city_state * CALAMITY_DEADLINESS
                                };
                                let new_death_prob = city_state * CALAMITY_DEADLINESS;
                                println!("next death prob for char {:?}: {:?}", id, new_death_prob);
                                let weight_update = [(0, &new_move_prob), (1, &new_death_prob)];
                                let update_result = character_states[id]
                                    .event_probability_map
                                    .update_weights(&weight_update);
                                match update_result {
                                    Ok(_) => (),
                                    Err(_) => println!("\t\t\tERR: Could not change character {:?}'s death probability", CharacterID(id))
                                }
                            }
                        }
                    }
                }
            }
            None => (),
        }
    }
}

pub struct World {
    pub cities: HashMap<CityID, City>,
    pub characters: HashMap<CharacterID, Character>,
    pub events: HashMap<EventID, Event>,
    city_id_counter: usize,
    pub event_id_counter: usize,
    character_id_counter: usize,
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
            layers: Default::default(),
        }
    }

    pub fn generate_world() -> Self {
        let mut world = World::new();
        println!("---- World Generation ----");

        println!("Generating cities...");

        // add end city
        world.layers[NUM_LAYERS - 1] = vec![world.add_city(NUM_LAYERS - 1)];

        // add in between cities - we will work from end to beginning so that each city is guaranteed to be connected to at least one city from the next layer
        for layer in (1..=NUM_LAYERS - 2).rev() {
            let num_cities =
                rand::thread_rng().gen_range(MIN_CITIES_IN_LAYER..=MAX_CITIES_IN_LAYER);

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

        println!("Generated {:?} cities", world.city_id_counter);

        // add characters
        for _ in (0..NUM_CHARACTERS) {
            world.add_character();
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
        let city1 = self.cities.get_mut(id1).unwrap();
        city1.neighbours.push(*id2);

        //let mut city2 = self.cities.get_mut(id2).unwrap();
        // city2.neighbours.push(*id1);
    }

    fn add_character(&mut self) -> CharacterID {
        let id = self.character_id_counter;
        self.character_id_counter += 1;
        let char = Character::new();
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
        summary: String,
    ) -> EventID {
        let event_id = EventID(self.event_id_counter);
        self.event_id_counter += 1;
        let event = Event::new(characters, start_time, end_time, event_type, summary);

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

    fn event_move(
        &mut self,
        time: usize,
        state: &mut CharacterState,
        rng: &mut ThreadRng,
        city_populations: &HashMap<CityID, Vec<CharacterID>>,
        city_calamity_states: &HashMap<CityID, usize>,
    ) {
        // determine city to move to
        let curr_city = self.cities.get(&state.city).unwrap();
        let &next_city = curr_city.neighbours.choose(rng).unwrap();

        // change character city to next city
        state.city = next_city;

        // recalculate probability map
        let population = city_populations.get(&next_city).unwrap().len();
        // if the character moves to the last city, set probability of moving again to zero.
        // otherwise, the probability is proportional to half the population of the city plus the city's calamity state.
        let next_city_id_num: usize = match next_city {
            CityID(id) => id,
        };
        let last_city_id: usize = 0;
        let next_city_calamity_state = city_calamity_states.get(&next_city).unwrap();
        let new_move_prob = if next_city_id_num == last_city_id {
            0
        } else {
            population / 4 + next_city_calamity_state * CALAMITY_DEADLINESS
        };
        let new_death_prob = next_city_calamity_state * CALAMITY_DEADLINESS;
        // update probabilities to the new city's context

        let weight_updates = [
            (0, &new_move_prob),
            (1, &new_death_prob),
            (2, &(population.pow(2))),
        ];
        let update_result = state.event_probability_map.update_weights(&weight_updates);
        match update_result {
            Ok(_) => (),
            Err(_) => {
                // if no actions are possible, character will stop forever
                state.event_probability_map = WeightedIndex::new([0, 0, 0, 1]).unwrap();
            }
        }

        // add event to character's events
        self.add_event(
            vec![state.character],
            time,
            None,
            EventType::EventMove,
            next_city,
            format!(
                "Character #{:?} moved to City #{:?}",
                state.character, next_city
            ),
        );
    }

    fn event_death(&mut self, time: usize, state: &mut CharacterState) {
        // set all probabilities to zero except EventIdle
        state.dead = true;
        state.event_probability_map = WeightedIndex::new(&[0, 0, 0, 1]).unwrap();

        // add death event
        self.add_event(
            vec![state.character],
            time,
            None,
            EventType::EventMove,
            state.city,
            format!(
                "Character #{:?} died in City #{:?}",
                state.character, state.city
            ),
        );
    }

    fn event_encounter(
        &mut self,
        time: usize,
        state: &mut CharacterState,
        rng: &mut ThreadRng,
        city_populations: &HashMap<CityID, Vec<CharacterID>>,
    ) {
        // pick random person from city to encounter
        let city_population = city_populations.get(&state.city).unwrap();
        let &encountered = city_population
            .iter()
            .filter(|&&id| id != state.character)
            .choose(rng)
            .unwrap();

        // add encounter event
        self.add_event(
            vec![state.character, encountered],
            time,
            None,
            EventType::EventEncounter,
            state.city,
            format!(
                "Character #{:?} encountered Character #{:?} in City #{:?}",
                state.character, encountered, state.city
            ),
        );

        // reduce probability of meeting after this to 0
        // (until character moves to a new city)
        let new_weights = [(2, &0)];
        let update_result = state.event_probability_map.update_weights(&new_weights);
        match update_result {
            Ok(_) => (),
            Err(_) => {
                // if no actions are possible, character will stop forever
                state.event_probability_map = WeightedIndex::new([0, 0, 0, 1]).unwrap();
            }
        }
    }

    // generates events chronologically and places them in the event lists of cities and characters.
    // a character only visits a city once and only encounters at most one other character in a city
    // run generate_world before running this or perish in the doomed worldless narrative that you've created
    pub fn generate_events(&mut self) {
        println!("---- Event Generation ----");

        // set up initial states for each character
        let mut states = Vec::new(); // in order of character id
        for char_id in 0..NUM_CHARACTERS {
            let starting_weights: [usize; NUM_EVENTS] = [
                cmp::max(NUM_CHARACTERS / 2, 1), // EventMove
                0,                               // EventDeath
                NUM_CHARACTERS, // EventEncounter (keep this proportional to the number of other characters in the same city)
                1,              // EventIdle
            ];
            let event_probability_map = WeightedIndex::new(&starting_weights).unwrap();
            states.push(CharacterState {
                character: CharacterID(char_id),
                city: self.layers[0][0], // start city
                event_probability_map,
                dead: false,
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

        fn recalculate_city_populations(
            cities: Vec<&CityID>,
            city_populations: &mut HashMap<CityID, Vec<CharacterID>>,
            states: &Vec<CharacterState>,
        ) {
            // reset and recalculate
            for &city_id in cities {
                let chars_in_city = get_characters_in_city(city_id, states);
                city_populations.insert(city_id, chars_in_city);
            }
        }

        // set up values for history
        let mut rng = thread_rng();
        let mut time = 0;
        let mut calamity_state = CalamityState::new(self.cities.keys().collect());
        // set initial city populations
        let mut city_populations: HashMap<CityID, Vec<CharacterID>> = HashMap::new();
        recalculate_city_populations(self.cities.keys().collect(), &mut city_populations, &states);

        println!("Generating events...");
        // start running history
        while time <= MAX_TIME {
            println!("time: {:?}", time);
            // step calamity movement
            calamity_state.calamity_step(
                time,
                &self.layers.to_vec(),
                &mut states,
                &city_populations,
            );

            // update each character's state
            for state_index in 0..states.len() {
                // determine next events for each character
                let state = &mut states[state_index];
                let next_event = &LIST_EVENTS[state.event_probability_map.sample(&mut rng)];

                // carry out event
                match next_event {
                    EventType::EventIdle => (), // do nothing (event idle is not logged)
                    EventType::EventMove => {
                        self.event_move(
                            time,
                            state,
                            &mut rng,
                            &city_populations,
                            &calamity_state.city_states,
                        );
                    }
                    EventType::EventDeath => {
                        self.event_death(time, state);
                    }
                    EventType::EventEncounter => {
                        self.event_encounter(time, state, &mut rng, &city_populations);
                    }
                }
                // recalculate city populations before next character acts
                recalculate_city_populations(
                    self.cities.keys().collect(),
                    &mut city_populations,
                    &states,
                );
            }
            time += 1;
        }
        println!("Generated {:?} events", self.event_id_counter);
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

const NAME_HARDLETTERS: &'static [&'static str] = &["p", "b", "t", "ch", "t", "k"];
const NAME_VOWELS: &'static [&'static str] = &["a", "e", "ae", "io", "ai", "u"];
const NAME_SOFTLETTERS: &'static [&'static str] = &["th", "nn", "ni", "sh"];

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

const PRONOUNS: &'static [&'static [&'static str]] = &[
    &["she", "her", "her"],
    &["him", "him", "his"],
    &["they", "them", "their"],
];

impl Character {
    fn name_gen() -> String {
        let mut first_syllable = "".to_string();
        let mut rng = rand::thread_rng();
        first_syllable.push_str(NAME_HARDLETTERS.choose(&mut rng).expect(""));
        first_syllable.push_str(NAME_VOWELS.choose(&mut rng).expect(""));
        first_syllable.push_str(NAME_SOFTLETTERS.choose(&mut rng).expect(""));
        first_syllable.push_str(NAME_VOWELS.choose(&mut rng).expect(""));

        first_syllable
    }

    fn pronoun_gen() -> Pronouns {
        let mut rng = rand::thread_rng();
        Pronouns {
            nominative: PRONOUNS[0].choose(&mut rng).unwrap().to_string(),
            accusative: PRONOUNS[0].choose(&mut rng).unwrap().to_string(),
            dep_genitive: PRONOUNS[0].choose(&mut rng).unwrap().to_string(),
        }
    }

    pub fn new() -> Self {
        Character {
            name: Self::name_gen(),
            pronouns: Self::pronoun_gen(),
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
    EventIdle, // an event representing doing nothing. this event should not be logged in event lists
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
    pub summary: String,
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
        summary: String,
    ) -> Self {
        Event {
            characters,
            start_time,
            end_time,
            event_type,
            events_happening_during: Vec::new(),
            summary,
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
