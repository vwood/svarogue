use super::{apply_point, spawner, Map, MapBuilder, Position, TileType};
use noise::{Fbm, NoiseFn, Seedable};
use rltk::RandomNumberGenerator;
use specs::prelude::*;
use std::cmp::{min, Ordering};
use std::collections::{BinaryHeap, HashSet};

/*
   Open question how to replace rooms...

   Could segment, create a list of potential spawn locations as we dig...

   or could use a poisson disc sampling method
*/

pub struct DigMapTurbBuilder {
    map: Map,
    starting_position: Position,
    depth: i32,
    spawn_candidates: Vec<Position>,
}

impl MapBuilder for DigMapTurbBuilder {
    fn get_map(&self) -> Map {
        self.map.clone()
    }

    fn get_starting_position(&self) -> Position {
        self.starting_position.clone()
    }

    fn build_map(&mut self) {
        // self.dig_map();
        self.dig_map_w_rooms();
    }

    fn spawn_entities(&mut self, ecs: &mut World) {
        spawner::spawn_locations(ecs, &self.spawn_candidates[..], self.depth);
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

#[derive(Copy, Clone, PartialEq)]
struct Room {
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
}

impl Room {
    pub fn overlap(&self, other: &Room) -> bool {
        !(self.x1 > other.x2 || self.y1 > other.y2 || self.y2 < other.y1 || self.x2 < other.x1)
    }

    pub fn in_bounds(&self, w: i32, h: i32) -> bool {
        self.x1 >= 0 && self.y1 >= 0 && self.x2 < w && self.y2 < h
    }
}

fn distance_to_centre(x: i32, y: i32, w: i32, h: i32) -> f64 {
    ((x - w / 2) as f64 / w as f64).powf(2.0) + ((y - h / 2) as f64 / h as f64).powf(2.0)
}

impl DigMapTurbBuilder {
    pub fn new(new_depth: i32) -> DigMapTurbBuilder {
        DigMapTurbBuilder {
            map: Map::new(new_depth),
            starting_position: Position { x: 0, y: 0 },
            depth: new_depth,
            spawn_candidates: Vec::new(),
        }
    }

    /// Use something approaching poisson sampling to find centre points
    /// to grow rooms
    fn poisson_sample_points(&mut self, count: i32) -> Vec<Position> {
        let (w, h) = (self.map.width, self.map.height);

        let mut result = Vec::new();
        let mut rng = RandomNumberGenerator::new();
        let mut candidates = Vec::new();
        for _ in 0..count * 3 {
            candidates.push(Position { x: rng.range(0, w), y: rng.range(0, h) })
        }

        let first = candidates.pop().unwrap();

        let mut distances = Vec::new();
        for candidate in &candidates {
            distances.push(first.square_distance(&candidate));
        }

        result.push(first);

        for _ in 0..count {
            let pick = (0..distances.len())
                .min_by_key(|i| &distances[*i as usize])
                .unwrap();

            let new = candidates.remove(pick);
            distances.remove(pick);

            for (i, candidate) in candidates.iter().enumerate() {
                let distance = new.square_distance(candidate);
                if distance < distances[i] {
                    distances[i] = distance;
                }
            }

            result.push(new);
        }

        result
    }

    /// Given a list of positions in the map, grow the rooms outwards
    fn grow_rooms(&mut self, seeds: Vec<Position>, probability: f64, iterations: i32) -> Vec<Room> {
        let (w, h) = (self.map.width, self.map.height);

        let mut rng = RandomNumberGenerator::new();
        let mut rooms = Vec::new();

        for position in seeds {
            rooms.push(Room {
                x1: position.x,
                y1: position.y,
                x2: position.x,
                y2: position.y,
            })
        }

        for _ in 0..iterations {
            for i in 0..rooms.len() {
                // TODO: find a way of collapsing this, can't access slots by name in structs
                if rng.range(0.0, 1.0) < probability {
                    rooms[i].x1 -= 1;

                    if !rooms[i].in_bounds(w, h) {
                        rooms[i].x1 += 1;
                        continue;
                    }
                    let mut bad = false;
                    for j in 0..rooms.len() {
                        if i != j && rooms[i].overlap(&rooms[j]) {
                            bad = true;
                            break;
                        }
                    }

                    if bad {
                        rooms[i].x1 += 1;
                    }
                }

                if rng.range(0.0, 1.0) < probability {
                    rooms[i].y1 -= 1;

                    if !rooms[i].in_bounds(w, h) {
                        rooms[i].y1 += 1;
                        continue;
                    }
                    let mut bad = false;
                    for j in 0..rooms.len() {
                        if i != j && rooms[i].overlap(&rooms[j]) {
                            bad = true;
                            break;
                        }
                    }

                    if bad {
                        rooms[i].y1 += 1;
                    }
                }

                if rng.range(0.0, 1.0) < probability {
                    rooms[i].x2 += 1;

                    if !rooms[i].in_bounds(w, h) {
                        rooms[i].x2 -= 1;
                        continue;
                    }
                    let mut bad = false;
                    for j in 0..rooms.len() {
                        if i != j && rooms[i].overlap(&rooms[j]) {
                            bad = true;
                            break;
                        }
                    }

                    if bad {
                        rooms[i].x2 -= 1;
                    }
                }

                if rng.range(0.0, 1.0) < probability {
                    rooms[i].y2 += 1;

                    if !rooms[i].in_bounds(w, h) {
                        rooms[i].y2 -= 1;
                        continue;
                    }
                    let mut bad = false;
                    for j in 0..rooms.len() {
                        if i != j && rooms[i].overlap(&rooms[j]) {
                            bad = true;
                            break;
                        }
                    }

                    if bad {
                        rooms[i].y2 -= 1;
                    }
                }
            }
        }

        rooms
    }

    /// Renders a list of rooms to an array so we can quickly test if we have hit a room,
    /// and which room we have hit.
    ///
    /// We tag room interiors as -2, and corners as -1
    /// normal space is 0, and walls get their own positive index based on the room_id
    /// (should probably use an enum or something)
    fn room_map(&mut self, rooms: &Vec<Room>) -> Vec<Vec<i32>> {
        let (w, h) = (self.map.width, self.map.height);
        let mut result = vec![vec![0i32; h as usize]; w as usize];

        for (i, room) in rooms.iter().enumerate() {
            let room_id = i as i32;
            for x in room.x1..min(w, room.x2 + 2) {
                for y in room.y1..min(h, room.y2 + 2) {
                    if x == room.x1 && (y == room.y1 || y == room.y2 + 1) {
                        result[x as usize][y as usize] = -1;
                    } else if x == room.x2 + 1 && (y == room.y1 || y == room.y2 + 1) {
                        result[x as usize][y as usize] = -1;
                    } else if x == room.x1 && x == 0 {
                        // no doors to edge of the map
                        result[x as usize][y as usize] = -1;
                    } else if y == room.y1 && y == 0 {
                        result[x as usize][y as usize] = -1;
                    } else if x == room.x1 {
                        result[x as usize][y as usize] = room_id * 4 + 1;
                    } else if x == room.x2 + 1 {
                        result[x as usize][y as usize] = room_id * 4 + 2;
                    } else if y == room.y1 {
                        result[x as usize][y as usize] = room_id * 4 + 3;
                    } else if y == room.y2 + 1 {
                        result[x as usize][y as usize] = room_id * 4 + 4;
                    } else {
                        result[x as usize][y as usize] = -2;
                    }
                }
            }
        }

        result
    }

    fn dig_map_w_rooms(&mut self) {
        const CONSTANT: f64 = -0.1;
        const EDGE_WEIGHT: f64 = 2.5;
        const ROOMS: i32 = 10;

        let seeds = self.poisson_sample_points(ROOMS);
        let rooms = self.grow_rooms(seeds, 0.8, 10);
        let room_array = self.room_map(&rooms);
        let mut walls_dug = HashSet::<i32>::new();

        let (w, h) = (self.map.width, self.map.height);
        let iterations = (w * h) / 2;

        let mut rng = RandomNumberGenerator::new();
        let simplex = Fbm::new().set_seed(rng.range(0, u32::MAX) as u32);
        let simplex2 = Fbm::new().set_seed(rng.range(0, u32::MAX) as u32);
        let turb_noise = |x, y| {
            let value = 8.0 * simplex.get([x, y]);

            simplex2.get([x + 4.0 * f64::sin(value), y + 4.0 * f64::cos(value)])
        };

        let mut heap = BinaryHeap::new();

        let start_x = rng.range(0, w / 2) as i32 + w / 4;
        let start_y = rng.range(0, h / 2) as i32 + h / 4;

        heap.push(Location { score: 0.0, x: start_x, y: start_y });

        let mut visited = vec![vec![0u8; h as usize]; w as usize];

        // constants for scaling the noise
        let (dx, dy) = (8.0 / w as f64, 8.0 / h as f64);

        let mut count = 0;
        for _ in 0..iterations {
            if let Some(Location { score, x, y }) = heap.pop() {
                let mut score = score;
                if visited[x as usize][y as usize] == 1 {
                    continue;
                }

                let room_id = room_array[x as usize][y as usize];
                if room_id == -1 {
                    continue;
                } else if room_id == -2 {
                    score -= 2.0; // Make room interiors much more likely to be carved out
                } else if room_id > 0 {
                    if walls_dug.contains(&room_id) {
                        continue;
                    }
                    walls_dug.insert(room_id);
                }

                count += 1;
                visited[x as usize][y as usize] = 1;

                if count % 40 == 0 {
                    self.spawn_candidates.push(Position { x: x, y: y });
                }

                let (fx, fy) = (x as f64 * dx, y as f64 * dy);

                if x > 0 && visited[x as usize - 1][y as usize] == 0 {
                    heap.push(Location {
                        score: score
                            + (turb_noise(fx - dx, fy) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x - 1,
                        y: y,
                    });
                }

                if x < w - 1 && visited[x as usize + 1][y as usize] == 0 {
                    heap.push(Location {
                        score: score
                            + (turb_noise(fx + dx, fy) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x + 1,
                        y: y,
                    });
                }

                if y > 0 && visited[x as usize][y as usize - 1] == 0 {
                    heap.push(Location {
                        score: score
                            + (turb_noise(fx, fy - dy) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x,
                        y: y - 1,
                    });
                }

                if y < h - 1 && visited[x as usize][y as usize + 1] == 0 {
                    heap.push(Location {
                        score: score
                            + (turb_noise(fx, fy + dy) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x,
                        y: y + 1,
                    });
                }

                if room_id > 0 {
                    // apply_point(&mut self.map, x, y, TileType::Door);
                    apply_point(&mut self.map, x, y, TileType::Floor);
                } else {
                    apply_point(&mut self.map, x, y, TileType::Floor);
                }
            }
        }

        for i in walls_dug {
            let room_id = ((i - 1) / 4) as usize;
            let room = rooms[room_id];

            for x in room.x1..min(w, room.x2 + 2) {
                for y in room.y1..min(h, room.y2 + 2) {
                    let idx = self.map.xy_idx(x, y);
                    if !(self.map.tiles[idx] == TileType::Door
                        || self.map.tiles[idx] == TileType::Floor)
                    {
                        if x == room.x1 || x == room.x2 + 1 || y == room.y1 || y == room.y2 + 1 {
                            self.map.tiles[idx] = TileType::StoneWall;
                        } else {
                            self.map.tiles[idx] = TileType::Floor;
                        }
                    }
                }
            }
        }

        let stairs_position = self.spawn_candidates.pop().expect("No spawn candidates!");
        apply_point(&mut self.map, stairs_position.x, stairs_position.y, TileType::DownStairs);

        self.starting_position = Position { x: start_x, y: start_y };
    }

    #[allow(dead_code)]
    fn dig_map(&mut self) {
        const CONSTANT: f64 = -0.1;
        const EDGE_WEIGHT: f64 = 2.5;

        let (w, h) = (self.map.width, self.map.height);
        let iterations = (w * h) / 2;

        let mut rng = RandomNumberGenerator::new();
        let simplex = Fbm::new().set_seed(rng.range(0, u32::MAX) as u32);
        let simplex2 = Fbm::new().set_seed(rng.range(0, u32::MAX) as u32);
        let turb_noise = |x, y| {
            let value = 8.0 * simplex.get([x, y]);

            simplex2.get([x + 4.0 * f64::sin(value), y + 4.0 * f64::cos(value)])
        };

        let mut heap = BinaryHeap::new();

        let start_x = rng.range(0, w / 2) as i32 + w / 4;
        let start_y = rng.range(0, h / 2) as i32 + h / 4;

        heap.push(Location { score: 0.0, x: start_x, y: start_y });

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

                if count % 40 == 0 {
                    self.spawn_candidates.push(Position { x: x, y: y });
                }

                let (fx, fy) = (x as f64 * dx, y as f64 * dy);

                if x > 0 && visited[x as usize - 1][y as usize] == 0 {
                    heap.push(Location {
                        score: score
                            + (turb_noise(fx - dx, fy) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x - 1,
                        y: y,
                    });
                }

                if x < w - 1 && visited[x as usize + 1][y as usize] == 0 {
                    heap.push(Location {
                        score: score
                            + (turb_noise(fx + dx, fy) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x + 1,
                        y: y,
                    });
                }

                if y > 0 && visited[x as usize][y as usize - 1] == 0 {
                    heap.push(Location {
                        score: score
                            + (turb_noise(fx, fy - dy) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x,
                        y: y - 1,
                    });
                }

                if y < h - 1 && visited[x as usize][y as usize + 1] == 0 {
                    heap.push(Location {
                        score: score
                            + (turb_noise(fx, fy + dy) + CONSTANT)
                            + EDGE_WEIGHT * distance_to_centre(x, y, w, h),
                        x: x,
                        y: y + 1,
                    });
                }

                apply_point(&mut self.map, x, y, TileType::Floor);
            }
        }

        let stairs_position = self.spawn_candidates.pop().expect("No spawn candidates!");
        let stairs_idx = self.map.xy_idx(stairs_position.x, stairs_position.y);
        self.map.tiles[stairs_idx] = TileType::DownStairs;

        self.starting_position = Position { x: start_x, y: start_y };
    }
}
