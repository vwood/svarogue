use super::{spawner, Map, Position, Rect, TileType};
use rltk::RandomNumberGenerator;
mod simple_map;
use simple_map::SimpleMapBuilder;
mod dig_map;
use dig_map::DigMapBuilder;
mod common;
use common::*;
use specs::prelude::*;

pub trait MapBuilder {
    fn build_map(&mut self);
    fn spawn_entities(&mut self, ecs: &mut World);
    fn get_map(&self) -> Map;
    fn get_starting_position(&self) -> Position;
}

pub fn random_builder(new_depth: i32) -> Box<dyn MapBuilder> {
    let mut rng = RandomNumberGenerator::new();

    match rng.range(0, 2) {
        0 => Box::new(SimpleMapBuilder::new(new_depth)),
        _ => Box::new(DigMapBuilder::new(new_depth)),
    }
}
