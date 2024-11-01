use crate::world::*;
use godot::classes::Node;
use godot::prelude::*;

const TEAPOT_DESCRIPTIONS: &'static [&'static str] = &["A teapot 1.", "A teapot 2.", "A teapot 3."];

const VASE_DESCRIPTIONS: &'static [&'static str] = &["A vase 1.", "A vase 2.", "A vase 3."];

const BRICABRAC_WEAR: &'static [&'static [&'static str]] = &[
    &[
        "It's cracked on {:?}.",
        "The {:?}, having been worn down with time and wear, seem almost invisible.",
        "You can hear something shaking around inside the {:?}.",
        "You can see this once being a striking part of someone's {:?}.",
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

#[godot_api]
impl History {
    #[func]
    fn generate_history(&mut self) {
        // generate events
        self.world.generate_events();

        let world_items = &self.world.items;

        let mut item_data_vec: Vec<Gd<ItemData>> = Vec::new();
        for (item_id, item) in world_items.into_iter() {
            item_data_vec.push(ItemData::new(
                "WIP".into(),
                array![GString::from("")],
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
struct ItemStory {
    #[var]
    lines: Array<GString>,
}

impl ItemStory {
    fn new(lines: Array<GString>) -> Gd<Self> {
        Gd::from_object(Self { lines })
    }
}
