extern crate strfmt;
use std::collections::HashMap;

use crate::world::*;
use godot::classes::Node;
use godot::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use strfmt::strfmt;

const MAX_WEAR_DESC: usize = 2;

// possible item descriptions
const TEAPOT_DESCRIPTIONS: &'static [&'static str] = &["A teapot 1.", "A teapot 2.", "A teapot 3."];
const VASE_DESCRIPTIONS: &'static [&'static str] = &["A vase 1.", "A vase 2.", "A vase 3."];

// possible wear descriptions
const BRICABRAC_WEAR: &'static [&'static [&'static str]] = &[
    &[
        "It's cracked on {0}.",
        "The {1}, having been worn down with time and wear, seem almost invisible.",
        "You can hear something shaking around inside the {2}.",
        "You can see this once being a striking part of someone's {3}.",
    ],
    &[
        "the handle",
        "ornamental colors",
        "porcelain",
        "kitchenware",
    ],
    &["the mouth", "embossings", "clay", "home decor"],
];
const ACCESORIES_WEAR: &'static[&'static[&'static str]] = &[
    &[
        "This was once an eye-catching statement, but time has only been as kind to it as its owners.",
        "It has an odd smell.",
        "Parts of it are... ash-y?"
    ],
    &[],
];

// possible introduction lines in a story
const STORY_INTROS: &'static [&'static str] = &["God, let me think... It's been a while."];

// possible event-specific lines for a story
const CREATION_LINES: &'static [&'static [&'static str]] = &[&[
    "Gosh, you're really testing my memory now...",
    "I remember one of my... returning customers, briefly remarking on the subject.",
    "They believed it was created by this person called {creator_name} way back in the day.",
    "As to why or how, your guess is as good as mine.",
]];
const DEATH_LINES: &'static [&'static [&'static str]] = &[&[]];
const MOVE_LINES: &'static [&'static [&'static str]] = &[&[]];
const EXCHANGE_LINES: &'static [&'static [&'static str]] = &[&[]];

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct History {
    world: World,
    generated: bool,
    items: Array<Gd<ItemData>>,
    base: Base<Node>,
}

fn get_item_types(item: &Item) -> (usize, usize, usize) {
    let item_type_string = item.item_type.to_string();
    let item_supertype_i: usize = match item.item_type {
        ItemType::Teapot1
        | ItemType::Teapot2
        | ItemType::Teapot3
        | ItemType::Vase1
        | ItemType::Vase2
        | ItemType::Vase3 => 0,
    };
    let item_type_i: usize = match item.item_type {
        ItemType::Teapot1 | ItemType::Teapot2 | ItemType::Teapot3 => 0,
        ItemType::Vase1 | ItemType::Vase2 | ItemType::Vase3 => 1,
    };
    let item_subtype_i: usize = item_type_string
        .chars()
        .last()
        .unwrap()
        .to_string()
        .parse()
        .unwrap();

    (item_supertype_i, item_type_i, item_subtype_i)
}

pub fn generate_description(item: &Item, item_types: (usize, usize, usize)) -> Array<GString> {
    let mut rng = rand::thread_rng();
    let wear_desc_amt = rng.gen_range(0..=MAX_WEAR_DESC);
    let mut description: Array<GString> = Array::new();

    // first desc
    let first_desc: GString = match item.item_type {
        ItemType::Teapot1 | ItemType::Teapot2 | ItemType::Teapot3 => {
            TEAPOT_DESCRIPTIONS[item_types.2]
        }
        ItemType::Vase1 | ItemType::Vase2 | ItemType::Vase3 => VASE_DESCRIPTIONS[item_types.2],
    }
    .into();
    description.push(first_desc);

    // desc of wear
    let wear_list = match item_types.0 {
        0 => BRICABRAC_WEAR,
        1 => ACCESORIES_WEAR,
        _ => unreachable!(),
    };
    let mut wear_format_hashmap: HashMap<String, String> = HashMap::new();
    let mut i: usize = 0;
    for &desc in wear_list[item_types.1 + 1] {
        wear_format_hashmap.insert(i.to_string(), desc.to_string());
        i += 1;
    }
    let wear_desc: Vec<GString> = wear_list[0]
        .choose_multiple(&mut rng, wear_desc_amt)
        .map(|&desc| GString::from(strfmt(desc, &wear_format_hashmap).unwrap()))
        .collect();
    description.extend(wear_desc);

    description
}

pub fn get_records_from_time(records: &Vec<ItemMoveRecord>, time: usize) -> Vec<&ItemMoveRecord> {
    let result: Vec<&ItemMoveRecord> = records
        .into_iter()
        .filter(|&record| record.time == time)
        .collect();

    result
}

pub fn generate_lines_from_event(world: &World, record: &ItemMoveRecord) -> Option<Array<GString>> {
    let event = world.events.get(&record.event.unwrap()).unwrap();
    match event.event_type {
        EventType::EventCreation(_) => {
            let &lines = CREATION_LINES.choose(&mut rand::thread_rng()).unwrap();
            let mut lines_gstring: Array<GString> = Array::new();
            for &line in lines {
                let owner = world.characters.get(&record.expect_owner()).unwrap();
                let mut format_vars: HashMap<String, String> = HashMap::new();
                format_vars.insert("creator_name".to_string(), owner.name.clone());
                let line_formatted = strfmt(line, &format_vars).unwrap();
                println!("Creation event line: {:?}", line_formatted);
                lines_gstring.push(&line_formatted.into());
            }
            Some(lines_gstring)
        }
        // EventType::EventDeath => Some(GString::new()),
        // EventType::EventMove => Some(GString::new()),
        // EventType::EventEncounter => Some(GString::new()),
        _ => None,
    }
}

pub fn generate_stories(world: &World, item: &Item) -> Array<Gd<ItemStory>> {
    let mut stories: Array<Gd<ItemStory>> = Array::new();
    let records = &item.owner_records;
    let oldest_records = get_records_from_time(&records, 0);
    println!("oldest records: {:?}", oldest_records);
    let last_time_seen = records.last().unwrap().time;
    let newest_records = get_records_from_time(&records, last_time_seen);
    println!(
        "newest records (last seen at time {:?}): {:?}",
        last_time_seen, newest_records
    );

    // generate oldest story, special dialogue for this
    let mut oldest_story_lines: Array<GString> = Array::new();
    // choose an intro
    let &intro = STORY_INTROS.choose(&mut rand::thread_rng()).unwrap();
    oldest_story_lines.push(&intro.into());
    // add rest
    oldest_story_lines
        .extend_array(&generate_lines_from_event(world, oldest_records.first().unwrap()).unwrap());
    // push to array of stories
    stories.push(ItemStory::new(oldest_story_lines));

    // generate other stories
    for record_i in (1..MAX_TIME) {
        ()
    }

    // generate newest story, special dialogue for this
    let mut newest_story_lines: Array<GString> = Array::new();

    // collect into array and return
    stories
}

#[godot_api]
impl History {
    #[func]
    fn generate_history(&mut self) {
        // generate events
        self.world.generate_events();

        let world_items = &self.world.items;
        let mut item_data_vec: Vec<Gd<ItemData>> = Vec::new();
        for (item_id, item) in world_items.into_iter() {
            let item_types = get_item_types(item);
            let item_type_string = item.item_type.to_string();
            let description = generate_description(item, item_types);

            item_data_vec.push(ItemData::new(
                item_type_string.into(),
                description,
                array![ItemStory::new(array![GString::from("")])],
            ));
        }

        self.items = array![];

        // replace this with actual history!
        self.items = array![ItemData::new(
            "shoe".into(),
            array![
                GString::from("A pair of shoes."),
                GString::from("The soles are worn out."),
                GString::from("They have an odd smell."),
                GString::from("The laces are burnt."),
            ],
            array![
                ItemStory::new(array![
                    GString::from("I bought these shoes off a guy named Jacque..."),
                    GString::from("Strange man."),
                    GString::from("He hailed from Nilte."),
                    GString::from("When the fire hit in 443, he fled to our city."),
                ]),
                ItemStory::new(array![
                    GString::from("Who had them before Jacque?"),
                    GString::from("Good question."),
                    GString::from("They were brought to Nilte in 411 by a woman called Helen from Troy."),
                    GString::from("She once stood them in pig shit."),
                    GString::from("They haven't smelt the same since."),
                    GString::from("I think she tried throwing them away, and Jacque picked them up for some reason.")
                ])
            ]
        )];

        self.generated = true;
    }

    #[func]
    fn get_item(&self, idx: i64) -> Option<Gd<ItemData>> {
        // early return if no item
        if !self.generated {
            return None;
        }

        self.items.get(idx as usize)
    }
}

#[godot_api]
impl INode for History {
    fn init(base: Base<Node>) -> Self {
        Self {
            world: World::generate_world(),
            generated: false,
            items: array![],
            base,
        }
    }
}

#[derive(GodotClass)]
#[class(no_init)]
struct ItemData {
    #[var]
    item_type: GString,
    #[var]
    description: Array<GString>,
    #[var]
    stories: Array<Gd<ItemStory>>,
}

#[godot_api]
impl ItemData {
    fn new(
        item_type: GString,
        description: Array<GString>,
        stories: Array<Gd<ItemStory>>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            item_type,
            description,
            stories,
        })
    }
}

#[derive(GodotClass)]
#[class(no_init)]
pub struct ItemStory {
    #[var]
    lines: Array<GString>,
}

impl ItemStory {
    fn new(lines: Array<GString>) -> Gd<Self> {
        Gd::from_object(Self { lines })
    }
}
