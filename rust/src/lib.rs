pub mod world;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_generator() {
        use crate::world::*;

        let mut world = world::World::generate_world();
        println!("{:?}", world.layers[0]);
        println!("{:?}", world.layers[1]);
        println!("{:?}", world.layers[2]);
        println!("{:?}", world.layers[3]);
        println!("{:?}", world.layers[4]);
        println!("{:?}", world.characters[&CharacterID(0)]);
        println!("{:?}", world.characters[&CharacterID(1)]);
        println!("{:?}", world.characters[&CharacterID(2)]);

        println!("{:?}", world.cities.get(&CityID(3)).unwrap().neighbours);
    }

    #[test]
    fn events() {
        use crate::world::*;

        // try creating some discrete events
        let test_event: Event =
            Event::new(vec![world::CharacterID(0)], 2, None, EventType::EventDeath);

        let test_event2: Event = Event::new(
            vec![world::CharacterID(0)],
            1,
            Some(4),
            EventType::EventDeath,
        );

        println!("{:?}", test_event);
        println!("{:?}", test_event2);

        // test_event.add_event_during(world, event_id);
    }
}
