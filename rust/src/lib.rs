pub mod world;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_generator() {
        use crate::world::*;
        
        let world = world::World::generate_world();
        println!("{:?}",world.layers[0]);
        println!("{:?}",world.layers[1]);
        println!("{:?}",world.layers[2]);
        println!("{:?}",world.layers[3]);
        println!("{:?}",world.layers[4]);
        println!("{:?}",world.characters[&CharacterID(0)]);
        println!("{:?}",world.characters[&CharacterID(1)]);
        println!("{:?}",world.characters[&CharacterID(2)]);
    
        while true {
            let mut city_id = String::new();
            
            std::io::stdin()
                .read_line(&mut city_id)
                .expect("Number should have been entered.");
        
            let city_id: u32 = city_id.trim().parse().expect("");
        
            println!("{:?}", world.cities.get(&CityID(city_id)).unwrap().neighbours);
            println!("{:?}", world.cities.get(&CityID(city_id)).unwrap().name);
        }
    }
}
