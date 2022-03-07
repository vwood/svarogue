use super::{
    apply_point, spawner, Map, MapBuilder,
    Position, Rect, TileType,
};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::collections::BinaryHeap;
use noise::{Fbm, NoiseFn, Seedable};
use std::cmp::Ordering;

/*
   Open question how to replace rooms... 

   Could segment, create a list of potential spawn locations as we dig...

   or could use a poisson disc sampling method
*/

pub struct DigMapBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    spawn_candidates: Vec<Position>,
}

impl MapBuilder for DigMapBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        self.dig_map();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        // TODO: solve this later
        
        // for room in self.rooms.iter().skip(1) {
        //    spawner::spawn_room(ecs, room, self.depth);
        // }
    }
}

/* Struct required for priority queue */
#[derive(Copy, Clone, PartialEq)]
struct Location {
    score: f64,
    x: i32,
    y: i32,
}

impl Eq for Location {}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .score
            .partial_cmp(&self.score)
            .unwrap_or(Ordering::Equal)
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(
            other
                .score
                .partial_cmp(&self.score)
                .unwrap_or(Ordering::Equal),
        )
    }
}

fn distance_to_centre(x: i32, y: i32, w: i32, h: i32) -> f64 {
    ((x - w / 2) as f64 / w as f64).powf(2.0) + ((y - h / 2) as f64 / h as f64).powf(2.0)
}

impl DigMapBuilder {
    pub fn new(new_depth: i32) -> DigMapBuilder {
        DigMapBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            spawn_candidates: Vec::new(),
        }
    }

    fn dig_map(&mut self) {
        const CONSTANT : f64 = -0.1;
        const EDGE_WEIGHT : f64 = 2.5;
        
        let (w, h) = (self.map.width, self.map.height);
        let iterations = (w * h) / 2;

        let mut rng = RandomNumberGenerator::new();
        let simplex = Fbm::new().set_seed(rng.range(0, u32::MAX) as u32);
        let mut heap = BinaryHeap::new();

        let start_x = rng.range(0, w / 2) as i32 + w / 4;
        let start_y = rng.range(0, h / 2) as i32 + h / 4;

        
        heap.push(Location {
            score: 0.0,
            x: start_x,
            y: start_y,
        });

        let mut visited = vec![vec![0u8; h as usize]; w as usize];

        // constants for scaling the noise
        let (dx, dy) = (8.0 / w as f64, 8.0 / h as f64);

        let mut count = 0;
        for _ in 0..iterations {
            if let Some(Location { score, x, y }) = heap.pop() {
                if visited[x as usize][y as usize] == 1 {
                    continue;
                }

                count += 1;
                visited[x as usize][y as usize] = 1;

                if count % 10 == 0 {
                    self.spawn_candidates.push(Position{ x: x, y: y});
                }
                
                let (fx, fy) = (x as f64 * dx, y as f64 * dy);

                if x > 0 && visited[x as usize - 1][y as usize] == 0 {
                    heap.push(Location {
                        score: score + (simplex.get([fx - dx, fy]) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x - 1,
                        y: y,
                    });
                }

                if x < w - 1 && visited[x as usize + 1][y as usize] == 0 {
                    heap.push(Location {
                        score: score + (simplex.get([fx + dx, fy]) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x + 1,
                        y: y,
                    });
                }

                if y > 0 && visited[x as usize][y as usize - 1] == 0 {
                    heap.push(Location {
                        score: score + (simplex.get([fx, fy - dy]) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x,
                        y: y - 1,
                    });
                }

                if y < h - 1 && visited[x as usize][y as usize + 1] == 0 {
                    heap.push(Location {
                        score: score + (simplex.get([fx, fy + dy]) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x,
                        y: y + 1,
                    });
                }

                apply_point(&mut self.map, x, y);
            }
        }

        let stairs_position = self.spawn_candidates.pop().expect("No spawn candidates!");
        let stairs_idx = self.map.xy_idx(stairs_position.x, stairs_position.y);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        self.starting_position = Position{ x: start_x, y: start_y};
    }
}
