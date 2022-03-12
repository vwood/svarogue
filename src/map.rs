use super::Position;
use noise::{Fbm, NoiseFn, Seedable};
use rltk::RandomNumberGenerator;
use rltk::{Algorithm2D, BaseMap, Point, Rltk, RGB};
use serde::{Deserialize, Serialize};
use specs::prelude::*;

pub const MAPWIDTH: usize = 80;
pub const MAPHEIGHT: usize = 43;
pub const MAPCOUNT: usize = MAPHEIGHT * MAPWIDTH;

#[derive(PartialEq, Copy, Clone, Serialize, Deserialize)]
pub enum TileType {
    DirtWall,
    DirtWall2,
    StoneWall,
    Floor,
    DownStairs,
    Door,
}

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Map {
    pub tiles: Vec<TileType>,
    pub width: i32,
    pub height: i32,
    pub revealed_tiles: Vec<bool>,
    pub visible_tiles: Vec<bool>,
    pub blocked: Vec<bool>,
    pub depth: i32,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    pub tile_content: Vec<Vec<Entity>>,
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn is_exit_valid(&self, x: i32, y: i32) -> bool {
        if x < 1 || x > self.width - 1 || y < 1 || y > self.height - 1 {
            return false;
        }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i, tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::StoneWall
                || *tile == TileType::DirtWall
                || *tile == TileType::DirtWall2;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    pub fn new(new_depth: i32) -> Map {
        let mut tiles = vec![TileType::DirtWall; MAPCOUNT];
        let mut rng = RandomNumberGenerator::new();

        let simplex = Fbm::new().set_seed(rng.range(0, u32::MAX) as u32);

        for x in 0..MAPWIDTH {
            for y in 0..MAPHEIGHT {
                let idx = (y as usize * MAPWIDTH) + x as usize;

                if simplex.get([
                    4.0 * x as f64 / MAPWIDTH as f64,
                    4.0 * y as f64 / MAPHEIGHT as f64,
                ]) > 0.0
                {
                    tiles[idx] = TileType::DirtWall2;
                }
            }
        }

        Map {
            tiles: tiles,
            width: MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles: vec![false; MAPCOUNT],
            visible_tiles: vec![false; MAPCOUNT],
            blocked: vec![false; MAPCOUNT],
            tile_content: vec![Vec::new(); MAPCOUNT],
            depth: new_depth,
        }
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx: usize) -> bool {
        self.tiles[idx] == TileType::DirtWall
            || self.tiles[idx] == TileType::StoneWall
            || self.tiles[idx] == TileType::DirtWall2
    }

    fn get_pathing_distance(&self, idx1: usize, idx2: usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx: usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x - 1, y) {
            exits.push((idx - 1, 1.0))
        };
        if self.is_exit_valid(x + 1, y) {
            exits.push((idx + 1, 1.0))
        };
        if self.is_exit_valid(x, y - 1) {
            exits.push((idx - w, 1.0))
        };
        if self.is_exit_valid(x, y + 1) {
            exits.push((idx + w, 1.0))
        };

        // Diagonals
        if self.is_exit_valid(x - 1, y - 1) {
            exits.push(((idx - w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y - 1) {
            exits.push(((idx - w) + 1, 1.45));
        }
        if self.is_exit_valid(x - 1, y + 1) {
            exits.push(((idx + w) - 1, 1.45));
        }
        if self.is_exit_valid(x + 1, y + 1) {
            exits.push(((idx + w) + 1, 1.45));
        }

        exits
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

pub fn draw_map(ecs: &World, ctx: &mut Rltk) {
    let map = ecs.fetch::<Map>();

    let mut y = 0;
    let mut x = 0;
    for (idx, tile) in map.tiles.iter().enumerate() {
        // Render a tile depending upon the tile type

        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(0.0, 0.8, 0.8);
                }
                TileType::StoneWall => {
                    glyph = 177; // rltk::to_cp437('#');
                    fg = RGB::from_f32(0.5, 0.6, 0.8);
                }
                TileType::DirtWall2 => {
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(0.8, 0.8, 0.4);
                }
                TileType::DirtWall => {
                    glyph = rltk::to_cp437('#');
                    fg = RGB::from_f32(0.8, 0.5, 0.2);
                }
                TileType::DownStairs => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0.0, 1.0, 1.0);
                }
                TileType::Door => {
                    glyph = rltk::to_cp437('+');
                    fg = RGB::from_f32(1.0, 0.2, 0.2);
                }
            }
            if !map.visible_tiles[idx] {
                fg = fg.to_greyscale() * 0.8
            }
            ctx.set(x, y, fg, RGB::from_f32(0., 0., 0.), glyph);
        }

        // Move the coordinates
        x += 1;
        if x > MAPWIDTH as i32 - 1 {
            x = 0;
            y += 1;
        }
    }
}

pub fn find_empty_adjacent(ecs: &World, x: i32, y: i32) -> (i32, i32) {
    let map = ecs.fetch::<Map>();
    let (w, h) = (map.width, map.height);

    let entities = ecs.entities();

    let possibilities = vec![
        (-1, 0),
        (1, 0),
        (0, 1),
        (0, -1),
        (-1, 1),
        (-1, -1),
        (1, -1),
        (1, 1),
    ];

    for (px, py) in possibilities {
        let (nx, ny) = (x + px, y + py);
        if nx < 0 || nx >= w || ny < 0 || ny >= h {
            continue;
        }

        let idx = map.xy_idx(nx, ny);
        if map.blocked[idx] {
            continue;
        }

        if map.tile_content[idx].len() > 0 {
            continue;
        }

        return (nx, ny);
    }

    return (x, y);
}
