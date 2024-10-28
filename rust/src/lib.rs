pub mod world;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_generator() {
        use crate::world::*;
        
        let mut world = World::generate_world();
        println!("{:?}",world.layers[0]);
        println!("{:?}",world.layers[1]);
        println!("{:?}",world.layers[2]);
        println!("{:?}",world.layers[3]);
        println!("{:?}",world.layers[4]);
    
        println!("{:?}", world.cities.get(&CityID(3)).unwrap().neighbours);
    }
}
