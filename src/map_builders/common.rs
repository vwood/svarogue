use super::{Map, Rect, TileType};
use std::cmp::{max, min};

pub fn apply_room_to_map(map: &mut Map, room: &Rect) {
    for y in room.y1 + 1..=room.y2 {
        for x in room.x1 + 1..=room.x2 {
            let idx = map.xy_idx(x, y);
            map.tiles[idx] = TileType::Floor;
        }
    }

    for y in room.y1..=room.y2 + 1 {
        let idx = map.xy_idx(room.x1, y);
        if map.tiles[idx] != TileType::Floor {
            map.tiles[idx] = TileType::StoneWall;
        }
        let idx2 = map.xy_idx(room.x2 + 1, y);
        if map.tiles[idx2] != TileType::Floor {
            map.tiles[idx2] = TileType::StoneWall;
        }
    }

    for x in room.x1..=room.x2 + 1 {
        let idx = map.xy_idx(x, room.y1);
        if map.tiles[idx] != TileType::Floor {
            map.tiles[idx] = TileType::StoneWall;
        }
        let idx2 = map.xy_idx(x, room.y2 + 1);
        if map.tiles[idx2] != TileType::Floor {
            map.tiles[idx2] = TileType::StoneWall;
        }
    }
}

pub fn apply_horizontal_tunnel(map: &mut Map, x1: i32, x2: i32, y: i32) {
    for x in min(x1, x2)..=max(x1, x2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize {
            map.tiles[idx as usize] = TileType::Floor;
        }
    }
}

pub fn apply_vertical_tunnel(map: &mut Map, y1: i32, y2: i32, x: i32) {
    for y in min(y1, y2)..=max(y1, y2) {
        let idx = map.xy_idx(x, y);
        if idx > 0 && idx < map.width as usize * map.height as usize {
            map.tiles[idx as usize] = TileType::Floor;
        }
    }
}

pub fn apply_point(map: &mut Map, x: i32, y: i32) {
    let idx = map.xy_idx(x, y);
    if idx > 0 && idx < map.width as usize * map.height as usize {
        map.tiles[idx as usize] = TileType::Floor;
    }
}
