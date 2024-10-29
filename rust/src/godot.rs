use godot::classes::Node;
use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(base=Node)]
struct History {
    generated: bool,
    items: Vec<ItemData>,
    base: Base<Node>,
}

#[godot_api]
impl History {
    #[func]
    fn generate_history(&mut self) {
        // generate history here
        self.generated = true;
    }
}

#[godot_api]
impl INode for History {
    fn init(base: Base<Node>) -> Self {
        Self {
            generated: false,
            items: vec![],
            base,
        }
    }
}

struct ItemData {
    item_type: ItemType,
    item_description: Vec<GString>,
    item_stories: Vec<Vec<GString>>,
}

enum ItemType {
    Type1,
    Type2,
    Type3,
}
