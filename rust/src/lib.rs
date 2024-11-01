pub mod godot;
pub mod world;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_generator() {
        use crate::world::*;

        let world = world::World::generate_world();
        println!("{:?}", world.layers[0]);
        println!("{:?}", world.layers[1]);
        println!("{:?}", world.layers[2]);
        println!("{:?}", world.layers[3]);
        println!("{:?}", world.layers[4]);
        println!("{:?}", world.characters[&CharacterID(0)]);
        println!("{:?}", world.characters[&CharacterID(1)]);
        println!("{:?}", world.characters[&CharacterID(2)]);

        /*
        while true {
            let mut city_id = String::new();

            std::io::stdin()
                .read_line(&mut city_id)
                .expect("Number should have been entered.");

            let city_id: u32 = city_id.trim().parse().expect("");

            println!(
                "{:?}",
                world.cities.get(&CityID(city_id)).unwrap().neighbours
            );
            println!("{:?}", world.cities.get(&CityID(city_id)).unwrap().name);
        }
        */
    }

    #[test]
    fn event_generator() {
        let mut world = world::World::generate_world();
        world.generate_events();
        println!("\nEvent Display:");
        for event_id in 0..world.event_id_counter {
            let &ref event = world.events.get(&world::EventID(event_id)).unwrap();
            println!("event {:?}: {:?}", event_id, event.summary);
        }
        for (_, character) in world.characters {
            println!("-------------------------");
            println!("Character {:?}'s events:", character.name);
            for event_id in character.events {
                let &ref event = world.events.get(&event_id).unwrap();
                println!("event #{:?}: {:?},", event_id, event.summary);
            }
        }
        for (item_id, item) in world.items {
            println!("-------------------------");
            println!(
                "Item #{:?} (of type {:?})'s events:",
                item_id, item.item_type
            );

            for record in item.owner_records {
                match record.event {
                    Some(event_id) => {
                        let &ref event = world.events.get(&event_id).unwrap();
                        println!("event #{:?}: {:?},", event_id, event.summary);
                    }
                    _ => (),
                }
            }
        }
    }

    #[test]
    fn run_eventgen_alot() {
        for _ in 0..2 {
            event_generator();
        }
    }

    #[test]
    fn text_gen() {
        let mut world = world::World::generate_world();
        world.generate_events();
        let test_item = world.items.get(&world::ItemID(0)).unwrap();
        godot::generate_stories(&world, test_item);
    }
}
