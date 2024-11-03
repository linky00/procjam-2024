extern crate strfmt;
use std::collections::HashMap;

use crate::world::*;
use godot::classes::Node;
use godot::prelude::*;
use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};
use strfmt::strfmt;

use std::fs::File;
use stringcase::Caser;

const MAX_WEAR_DESC: usize = 2;

#[derive(Serialize, Deserialize)]
struct WearDescs {
    bricabrac_wear: Vec<Vec<String>>,
    accessory_wear: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct EventLines {
    creation_lines: Vec<Vec<String>>,
    death_lines: Vec<Vec<String>>,
    move_lines: Vec<Vec<String>>,
    exchange_lines: Vec<Vec<String>>,
    postmortem_exchange_lines: Vec<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
pub struct DescJson {
    initial_descriptions: HashMap<String, String>,
    wear_descriptions: WearDescs,
    story_intros: Vec<String>,
    story_outros: Vec<String>,
    event_lines: EventLines,
}

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
        | ItemType::Vase3
        | ItemType::Cup1
        | ItemType::Statue
        | ItemType::Orb => 0,

        ItemType::Belt
        | ItemType::Bracelet
        | ItemType::Hat
        | ItemType::Shoes1
        | ItemType::Shoes2
        | ItemType::Shoes3
        | ItemType::Sunglasses
        | ItemType::Necklace => 1,
    };
    let item_type_i: usize = match item.item_type {
        ItemType::Teapot1 | ItemType::Teapot2 | ItemType::Teapot3 => 0,
        ItemType::Vase1 | ItemType::Vase2 | ItemType::Vase3 => 1,
        ItemType::Cup1 => 2,
        ItemType::Statue => 3,
        ItemType::Orb => 4,
        ItemType::Belt => 5,
        ItemType::Bracelet => 6,
        ItemType::Hat => 7,
        ItemType::Shoes1 | ItemType::Shoes2 | ItemType::Shoes3 => 8,
        ItemType::Sunglasses => 9,
        ItemType::Necklace => 10,
        _ => 0,
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

pub fn generate_description(
    item: &Item,
    item_types: (usize, usize, usize),
    descs: &DescJson,
) -> Array<GString> {
    let mut rng = rand::thread_rng();
    let wear_desc_amt = rng.gen_range(0..=MAX_WEAR_DESC);
    let mut description: Array<GString> = Array::new();

    let first_desc: GString = descs.initial_descriptions[&item.item_type.to_string()]
        .to_string()
        .into();
    description.push(first_desc);

    // desc of wear
    let wear_descs = &descs.wear_descriptions;
    let wear_list = match item_types.0 {
        0 => &wear_descs.bricabrac_wear,
        1 => &wear_descs.accessory_wear,
        _ => unreachable!(),
    };
    let mut wear_format_hashmap: HashMap<String, String> = HashMap::new();
    let mut i: usize = 0;
    for desc in wear_list
        .get(item_types.1 + 1)
        .expect("the appropriate non-formatted wear description")
        .iter()
    {
        wear_format_hashmap.insert(i.to_string(), desc.to_string());
        i += 1;
    }
    let wear_desc: Vec<GString> = wear_list[0]
        .choose_multiple(&mut rng, wear_desc_amt)
        .map(|desc| {
            GString::from(
                strfmt(desc, &wear_format_hashmap)
                    .expect("an appropriate wear description that's been formatted"),
            )
        })
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
    lines: &Vec<String>,
    world: &World,
    record: &ItemMoveRecord,
) -> Array<GString> {
    let event = world
        .events
        .get(&record.expect_event())
        .expect("the event obj associated with the given record");
    let mut lines_gstring: Array<GString> = Array::new();

    // get format parameters ready
    let mut format_vars: HashMap<String, String> = HashMap::new();
    // insert owner name
    let owner = world
        .characters
        .get(&record.expect_owner())
        .expect("reference to the character obj of the owner of the item with the given record");
    format_vars.insert(
        "owner_name".to_string(),
        owner.name.clone().to_pascal_case(),
    );
    // insert city name
    let city = world
        .cities
        .get(&record.expect_location())
        .expect("reference to the city obj of the given record");
    format_vars.insert("city_name".to_string(), city.name.clone().to_pascal_case());
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
            .expect("reference to the character object of the old owner of the item associated with the given record");
        format_vars.insert(
            "old_owner_name".to_string(),
            old_owner.name.clone().to_pascal_case(),
        );
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
    // add year
    format_vars.insert("year".to_string(), event.start_time.to_string());
    // format all lines
    for line in lines {
        // format line
        let line_formatted = strfmt(line, &format_vars)
            .expect("one of the formatted lines of the story generated for this event");
        println!("Creation event line: {:?}", line_formatted);
        lines_gstring.push(&line_formatted.into());
    }

    // return formatted lines
    lines_gstring
}

pub fn generate_lines_from_event(
    world: &World,
    record: &ItemMoveRecord,
    descs: &DescJson,
) -> Option<Array<GString>> {
    let event = world
        .events
        .get(
            &record
                .event
                .expect("the event id of the event associated with the given record"),
        )
        .expect("the event obj of the event associated with the given record");
    match event.event_type {
        EventType::EventCreation(_) => {
            let lines = descs
                .event_lines
                .creation_lines
                .choose(&mut rand::thread_rng())
                .expect("randomly chosen creation line");
            Some(format_event_lines(lines, world, record))
        }
        EventType::EventDeath => {
            let lines = descs
                .event_lines
                .death_lines
                .choose(&mut rand::thread_rng())
                .expect("randomly chosen death line");
            Some(format_event_lines(lines, world, record))
        }
        EventType::EventMove => {
            let lines = descs
                .event_lines
                .move_lines
                .choose(&mut rand::thread_rng())
                .expect("randomly chosen move line");
            Some(format_event_lines(lines, world, record))
        }
        EventType::EventEncounter => {
            let encountered_id = event
                .characters
                .get(1)
                .expect("the id of the character that is encountered");
            let is_postmortem_encounter = world
                .characters
                .get(encountered_id)
                .expect("the character struct for the encountered character")
                .has_died(world);
            let const_lines = if is_postmortem_encounter {
                &descs.event_lines.postmortem_exchange_lines
            } else {
                &descs.event_lines.exchange_lines
            };
            let lines = const_lines
                .choose(&mut rand::thread_rng())
                .expect("randomly chosen exchange line");
            Some(format_event_lines(lines, world, record))
        }
        _ => None,
    }
}

pub fn generate_stories(world: &World, item: &Item, descs: &DescJson) -> Array<Gd<ItemStory>> {
    let mut stories: Array<Gd<ItemStory>> = Array::new();
    let records = &item.owner_records;
    let oldest_records = get_records_from_time(&records, 0);
    println!("oldest records: {:?}", oldest_records);
    let last_time_seen = records
        .last()
        .expect("the last record in the owner records of this item")
        .time;
    let newest_records = get_records_from_time(&records, last_time_seen);

    // generate oldest story, special dialogue for this
    let mut oldest_story_lines: Array<GString> = Array::new();
    // add lines for event
    oldest_story_lines.extend_array(
        &generate_lines_from_event(
            world,
            oldest_records
                .first()
                .expect("oldest record pertaining to this item"),
            descs,
        )
        .expect("lines generated for the oldest record associated with the given item"),
    );
    // choose an outro
    let outro = descs
        .story_outros
        .choose(&mut rand::thread_rng())
        .expect("randomly chosen story outro");
    oldest_story_lines.push(&outro.into());
    // push to array of stories
    stories.push(ItemStory::new(oldest_story_lines));

    // generate in between stories
    for record_i in 1..(records.len() - 1) {
        let record = &records[record_i];
        let lines_option = generate_lines_from_event(world, &record, descs);
        match lines_option {
            Some(lines) => stories.push(ItemStory::new(lines)),
            None => (),
        }
    }

    // generate newest story, special dialogue for this
    let mut newest_story_lines: Array<GString> = Array::new();
    // choose an intro
    let intro = descs
        .story_intros
        .choose(&mut rand::thread_rng())
        .expect("randomly chosen story intro");
    newest_story_lines.push(&intro.into());
    // add lines for event
    newest_story_lines.extend_array(
        &generate_lines_from_event(
            world,
            newest_records
                .last()
                .expect("newest record associated with the given item"),
            &descs,
        )
        .expect("lines generated for the newest record associated with the given ite"),
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
        godot_print!("Generating events...");
        self.world.generate_events();
        godot_print!("Done generating events");

        // print item events for debugging
        let world_items = &self.world.items;
        godot_print!("\nItem Event Display:");
        for (item_id, item) in world_items {
            godot_print!("-------------------------");
            godot_print!(
                "Item #{:?} (of type {:?})'s events:",
                item_id,
                item.item_type
            );

            for record in &item.owner_records {
                match record.event {
                    Some(event_id) => {
                        let &ref event = self.world.events.get(&event_id).unwrap();
                        godot_print!("event #{:?}: {:?},", event_id, event.summary);
                    }
                    _ => (),
                }
            }
        }

        // generate item data for each item
        let desc_file = File::open("writing/descriptions.json").expect("opening descriptions file");
        let descs: DescJson = serde_json::from_reader(desc_file).unwrap();
        let mut item_data: Array<Gd<ItemData>> = Array::new();
        for (_, item) in world_items.into_iter() {
            let item_types = get_item_types(item);
            let item_type_string = item.item_type.to_string();
            let description = generate_description(item, item_types, &descs);
            let stories = generate_stories(&self.world, item, &descs);

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
