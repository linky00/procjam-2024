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
        // replacements for teapot
        "the handle",
        "ornamental colors",
        "porcelain",
        "kitchenware",
    ],
    &[
        // replacements for vase
        "the mouth",
        "embossings",
        "clay",
        "home decor",
    ],
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
const STORY_INTROS: &'static [&'static str] = &["Let me think... It's been a while."];
// possible ending lines in a story
const STORY_OUTROS: &'static [&'static str] = &["I believe that's all that's known about that."];

// possible event-specific lines for a story
// FORMAT RULES:
// write {owner_name} as a stand-in for the item's owner's name.
// write {city_name} as a stand-in for the name of the city where item is.
// write {nominative_pronoun} as a stand-in for a nominative pronoun for the owner (she, he, they...).
// write {accusative_pronoun} as a stand-in for an accusative pronoun for the owner (her, him, them...).
// write {dep_genitive_pronoun} as a stand-in for a dependent genitive pronoun for the owner (her, his, their...).
// write {old_owner_name} when applicable as a stand-in for the old owner during an item exchange event.
// add 1 to the end of the pronoun placeholder as a stand-in for the pronouns of the old owner during an exchange event.
// ex: {nominative_pronoun1} would be the nominative pronoun of the old owner.
const CREATION_LINES: &'static [&'static [&'static str]] = &[&[
    "Gosh, you're really testing my memory now...",
    "I remember one of my... returning customers, briefly remarking on the subject.",
    "They believed it was created by this person called {owner_name} way back in the day, in {city_name}.",
    "As to why or how, your guess is as good as mine.",
]];
//
const DEATH_LINES: &'static [&'static [&'static str]] = &[&[
    "I think {nominative_pronoun} died in {city_name}, when the fires first hit.",
    "What a shame. Had {nominative_pronoun} survived, I'm certain we'd have much to tell each other.",
]];
const MOVE_LINES: &'static [&'static [&'static str]] = &[&[
    "Like most people who could afford to, {nominative_pronoun} moved to {city_name} when the calamity eventually hit.",
]];
const EXCHANGE_LINES: &'static [&'static [&'static str]] = &[&[
    "{owner_name} says {nominative_pronoun} stole it from someone called \"{old_owner_name}\" while they were both sheltering inside an old, broken down mill on the outskirts of {city_name}.",
    "I believe {accusative_pronoun}. They apparently slept together for a couple of days.",
    "I was told {nominative_pronoun} eventually decided to head away on some ungodly hour, and never saw {accusative_pronoun1} again in {dep_genitive_pronoun} waking life."
]];

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
    let mut item_subtype_i: usize = item_type_string
        .chars()
        .last()
        .unwrap()
        .to_string()
        .parse()
        .unwrap();
    item_subtype_i -= 1;

    (item_supertype_i, item_type_i, item_subtype_i)
}

pub fn generate_description(item: &Item, item_types: (usize, usize, usize)) -> Array<GString> {
    let mut rng = rand::thread_rng();
    let wear_desc_amt = rng.gen_range(0..=MAX_WEAR_DESC);
    let mut description: Array<GString> = Array::new();

    // first desc
    let first_desc: GString = match item.item_type {
        ItemType::Teapot1 | ItemType::Teapot2 | ItemType::Teapot3 => {
            TEAPOT_DESCRIPTIONS.get(item_types.2).expect("line 131")
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
    for &desc in wear_list.get(item_types.1 + 1).expect("line 146").iter() {
        wear_format_hashmap.insert(i.to_string(), desc.to_string());
        i += 1;
    }
    let wear_desc: Vec<GString> = wear_list[0]
        .choose_multiple(&mut rng, wear_desc_amt)
        .map(|&desc| GString::from(strfmt(desc, &wear_format_hashmap).expect("line 152")))
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

pub fn format_event_lines(
    lines: &'static [&'static str],
    world: &World,
    record: &ItemMoveRecord,
) -> Array<GString> {
    let event = world.events.get(&record.expect_event()).expect("line 173");
    let mut lines_gstring: Array<GString> = Array::new();

    // get format parameters ready
    let mut format_vars: HashMap<String, String> = HashMap::new();
    // insert owner name
    let owner = world
        .characters
        .get(&record.expect_owner())
        .expect("line 179");
    format_vars.insert("owner_name".to_string(), owner.name.clone());
    // insert city name
    let city = world
        .cities
        .get(&record.expect_location())
        .expect("line 182");
    format_vars.insert("city_name".to_string(), city.name.clone());
    // insert pronouns of owner
    format_vars.insert(
        "nominative_pronoun".to_string(),
        owner.pronouns.nominative.clone(),
    );
    format_vars.insert(
        "accusative_pronoun".to_string(),
        owner.pronouns.accusative.clone(),
    );
    format_vars.insert(
        "dep_genitive_pronoun".to_string(),
        owner.pronouns.dep_genitive.clone(),
    );
    // add old owner info if applicable.
    if event.event_type == EventType::EventEncounter {
        let old_owner = world
            .characters
            .get(&event.characters[1])
            .expect("line 205");
        format_vars.insert("old_owner_name".to_string(), old_owner.name.clone());
        format_vars.insert(
            "nominative_pronoun1".to_string(),
            old_owner.pronouns.nominative.clone(),
        );
        format_vars.insert(
            "accusative_pronoun1".to_string(),
            old_owner.pronouns.accusative.clone(),
        );
        format_vars.insert(
            "dep_genitive_pronoun1".to_string(),
            old_owner.pronouns.dep_genitive.clone(),
        );
    }
    // add old city info if applicable.
    if event.event_type == EventType::EventMove {
        // TODO
    }

    // format all lines
    for &line in lines {
        // format line
        let line_formatted = strfmt(line, &format_vars).expect("line 231");
        println!("Creation event line: {:?}", line_formatted);
        lines_gstring.push(&line_formatted.into());
    }

    // return formatted lines
    lines_gstring
}

pub fn generate_lines_from_event(world: &World, record: &ItemMoveRecord) -> Option<Array<GString>> {
    let event = world
        .events
        .get(&record.event.expect("line 241"))
        .expect("line 241");
    match event.event_type {
        EventType::EventCreation(_) => {
            let &lines = CREATION_LINES
                .choose(&mut rand::thread_rng())
                .expect("line 247");
            Some(format_event_lines(lines, world, record))
        }
        EventType::EventDeath => {
            let &lines = DEATH_LINES
                .choose(&mut rand::thread_rng())
                .expect("line 253");
            Some(format_event_lines(lines, world, record))
        }
        EventType::EventMove => {
            let &lines = MOVE_LINES
                .choose(&mut rand::thread_rng())
                .expect("line 259");
            Some(format_event_lines(lines, world, record))
        }
        EventType::EventEncounter => {
            let &lines = EXCHANGE_LINES
                .choose(&mut rand::thread_rng())
                .expect("line 265");
            Some(format_event_lines(lines, world, record))
        }
        _ => None,
    }
}

pub fn generate_stories(world: &World, item: &Item) -> Array<Gd<ItemStory>> {
    let mut stories: Array<Gd<ItemStory>> = Array::new();
    let records = &item.owner_records;
    let oldest_records = get_records_from_time(&records, 0);
    println!("oldest records: {:?}", oldest_records);
    let last_time_seen = records.last().expect("line 279").time;
    let newest_records = get_records_from_time(&records, last_time_seen);
    println!(
        "newest records (last seen at time {:?}): {:?}",
        last_time_seen, newest_records
    );

    // generate oldest story, special dialogue for this
    let mut oldest_story_lines: Array<GString> = Array::new();
    // add lines for event
    oldest_story_lines.extend_array(
        &generate_lines_from_event(world, oldest_records.first().expect("line 290"))
            .expect("line 290"),
    );
    // choose an outro
    let &outro = STORY_OUTROS
        .choose(&mut rand::thread_rng())
        .expect("line 294");
    oldest_story_lines.push(&outro.into());
    // push to array of stories
    stories.push(ItemStory::new(oldest_story_lines));

    // generate in between stories
    for record_i in 1..last_time_seen {
        let record = &records[record_i];
        let lines_option = generate_lines_from_event(world, &record);
        match lines_option {
            Some(lines) => stories.push(ItemStory::new(lines)),
            None => (),
        }
    }

    // generate newest story, special dialogue for this
    let mut newest_story_lines: Array<GString> = Array::new();
    // choose an intro
    let &intro = STORY_INTROS
        .choose(&mut rand::thread_rng())
        .expect("line 314");
    newest_story_lines.push(&intro.into());
    // add lines for event
    newest_story_lines.extend_array(
        &generate_lines_from_event(world, newest_records.last().expect("line 320"))
            .expect("line 320"),
    );
    // push to array of stories
    stories.push(ItemStory::new(newest_story_lines));

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

        // generate item data for each item
        let mut item_data: Array<Gd<ItemData>> = Array::new();
        for (_, item) in world_items.into_iter() {
            let item_types = get_item_types(item);
            let item_type_string = item.item_type.to_string();
            let description = generate_description(item, item_types);
            let stories = generate_stories(&self.world, item);

            item_data.push(ItemData::new(item_type_string.into(), description, stories));
        }

        self.items = item_data;

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

/*
putting Ada's example item story down here for reference:
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
*/
