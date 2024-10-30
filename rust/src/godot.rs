use godot::classes::Node;
use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct History {
    generated: bool,
    items: Array<Gd<ItemData>>,
    base: Base<Node>,
}

#[godot_api]
impl History {
    #[func]
    fn generate_history(&mut self) {
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
    item_description: Array<GString>,
    #[var]
    item_stories: Array<Gd<ItemStory>>,
}

#[godot_api]
impl ItemData {
    fn new(
        item_type: GString,
        item_description: Array<GString>,
        item_stories: Array<Gd<ItemStory>>,
    ) -> Gd<Self> {
        Gd::from_object(Self {
            item_type,
            item_description,
            item_stories,
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
