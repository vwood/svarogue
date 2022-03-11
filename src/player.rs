use super::{
    gamelog::GameLog, CombatStats, EntityMoved, Item, Map, Monster, Player, Position, RunState,
    State, TileType, Viewshed, WantsToMelee, WantsToPickupItem,
};
use rltk::{Point, Rltk, VirtualKeyCode};
use specs::prelude::*;
use std::cmp::{max, min};

pub fn try_move_player(delta_x: i32, delta_y: i32, ecs: &mut World) {
    let mut positions = ecs.write_storage::<Position>();
    let players = ecs.read_storage::<Player>();
    let mut viewsheds = ecs.write_storage::<Viewshed>();
    let entities = ecs.entities();
    let combat_stats = ecs.read_storage::<CombatStats>();
    let map = ecs.fetch::<Map>();
    let mut wants_to_melee = ecs.write_storage::<WantsToMelee>();
    let mut entity_moved = ecs.write_storage::<EntityMoved>();

    for (entity, _player, pos, viewshed) in
        (&entities, &players, &mut positions, &mut viewsheds).join()
    {
        if pos.x + delta_x < 0
            || pos.x + delta_x > map.width - 1
            || pos.y + delta_y < 0
            || pos.y + delta_y > map.height - 1
        {
            return;
        }
        let destination_idx = map.xy_idx(pos.x + delta_x, pos.y + delta_y);

        for potential_target in map.tile_content[destination_idx].iter() {
            let target = combat_stats.get(*potential_target);
            if let Some(_target) = target {
                wants_to_melee
                    .insert(entity, WantsToMelee { target: *potential_target })
                    .expect("Add target failed");
                return;
            }
        }

        if !map.blocked[destination_idx] {
            pos.x = min(79, max(0, pos.x + delta_x));
            pos.y = min(49, max(0, pos.y + delta_y));

            viewshed.dirty = true;
            let mut ppos = ecs.write_resource::<Point>();
            ppos.x = pos.x;
            ppos.y = pos.y;
            entity_moved
                .insert(entity, EntityMoved {})
                .expect("Unable to insert marker");
        }
    }
}

pub fn try_next_level(ecs: &mut World) -> bool {
    let player_pos = ecs.fetch::<Point>();
    let map = ecs.fetch::<Map>();
    let player_idx = map.xy_idx(player_pos.x, player_pos.y);
    if map.tiles[player_idx] == TileType::DownStairs {
        true
    } else {
        let mut gamelog = ecs.fetch_mut::<GameLog>();
        gamelog
            .entries
            .push("There is no way down from here.".to_string());
        false
    }
}

fn get_item(ecs: &mut World) {
    let player_pos = ecs.fetch::<Point>();
    let player_entity = ecs.fetch::<Entity>();
    let entities = ecs.entities();
    let items = ecs.read_storage::<Item>();
    let positions = ecs.read_storage::<Position>();
    let mut gamelog = ecs.fetch_mut::<GameLog>();

    let mut target_item: Option<Entity> = None;
    for (item_entity, _item, position) in (&entities, &items, &positions).join() {
        if position.x == player_pos.x && position.y == player_pos.y {
            target_item = Some(item_entity);
        }
    }

    match target_item {
        None => gamelog
            .entries
            .push("There is nothing here to pick up.".to_string()),
        Some(item) => {
            let mut pickup = ecs.write_storage::<WantsToPickupItem>();
            pickup
                .insert(*player_entity, WantsToPickupItem { collected_by: *player_entity, item })
                .expect("Unable to insert want to pickup");
        }
    }
}

fn skip_turn(ecs: &mut World) -> RunState {
    let player_entity = ecs.fetch::<Entity>();
    let viewshed_components = ecs.read_storage::<Viewshed>();
    let monsters = ecs.read_storage::<Monster>();

    let worldmap_resource = ecs.fetch::<Map>();

    let mut can_heal = true;
    let viewshed = viewshed_components.get(*player_entity).unwrap();
    for tile in viewshed.visible_tiles.iter() {
        let idx = worldmap_resource.xy_idx(tile.x, tile.y);
        for entity_id in worldmap_resource.tile_content[idx].iter() {
            let mob = monsters.get(*entity_id);
            match mob {
                None => {}
                Some(_) => {
                    can_heal = false;
                }
            }
        }
    }

    let mut health_components = ecs.write_storage::<CombatStats>();
    let player_stats = health_components.get_mut(*player_entity).unwrap();
    if can_heal {
        player_stats.hp = i32::min(player_stats.hp + 1, player_stats.max_hp);
    }
    player_stats.stamina = i32::min(player_stats.stamina + 1, player_stats.max_stamina);

    RunState::PlayerTurn
}

pub fn player_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement
    match ctx.key {
        None => return RunState::AwaitingInput, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                try_move_player(-1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                try_move_player(1, 0, &mut gs.ecs)
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                try_move_player(0, -1, &mut gs.ecs)
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                try_move_player(0, 1, &mut gs.ecs)
            }

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => try_move_player(1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => try_move_player(-1, -1, &mut gs.ecs),

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => try_move_player(1, 1, &mut gs.ecs),

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => try_move_player(-1, 1, &mut gs.ecs),

            // Skip Turn
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),

            // Level changes
            VirtualKeyCode::Period => {
                if try_next_level(&mut gs.ecs) {
                    return RunState::NextLevel;
                }
            }

            // Picking up items
            VirtualKeyCode::G | VirtualKeyCode::Comma => get_item(&mut gs.ecs),
            VirtualKeyCode::I => return RunState::ShowInventory,
            VirtualKeyCode::D => return RunState::ShowDropItem,
            VirtualKeyCode::R => return RunState::ShowRemoveItem,

            // Cheat codes
            VirtualKeyCode::F2 => {
                let mut map = gs.ecs.fetch_mut::<Map>();
                for r in map.revealed_tiles.iter_mut() {
                    *r = true;
                }
            }

            // Moving weapons
            VirtualKeyCode::A => return RunState::MoveWeapon,
            VirtualKeyCode::S => return RunState::MoveShield,
            VirtualKeyCode::Z => return RunState::Dodge,

            VirtualKeyCode::F1 => {
                let mut gamelog = gs.ecs.fetch_mut::<GameLog>();
                gamelog
                    .entries
                    .push("'A' to move weapon, 'S' to move shield".to_string());
                gamelog.entries.push("'Z' to dodge".to_string());
                return RunState::AwaitingInput;
            }

            // Save and Quit
            VirtualKeyCode::Escape => return RunState::SaveGame,

            _ => return RunState::AwaitingInput,
        },
    }
    RunState::PlayerTurn
}

fn player_use_stamina(ecs: &mut World, amount: i32) -> bool {
    let mut gamelog = ecs.fetch_mut::<GameLog>();
    gamelog.entries.push("You exert yourself.".to_string());

    let player_entity = ecs.fetch::<Entity>();
    let mut combat_stats = ecs.write_storage::<CombatStats>();
    let player_stats = combat_stats.get_mut(*player_entity).unwrap();

    if player_stats.stamina < amount {
        false
    } else {
        player_stats.stamina = player_stats.stamina - amount;
        true
    }
}

///
/// WEAPON MOVEMENT SYSTEM
///
pub fn player_weapon_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement - weapon movement mode
    match ctx.key {
        None => return RunState::Dodge, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(-1, 0, &mut gs.ecs);
                    try_move_player(-1, 0, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(1, 0, &mut gs.ecs);
                    try_move_player(1, 0, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(0, -1, &mut gs.ecs);
                    try_move_player(0, -1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(0, 1, &mut gs.ecs);
                    try_move_player(0, 1, &mut gs.ecs);
                }
            }

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(1, -1, &mut gs.ecs);
                    try_move_player(1, -1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(-1, -1, &mut gs.ecs);
                    try_move_player(-1, -1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(1, 1, &mut gs.ecs);
                    try_move_player(1, 1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(-1, 1, &mut gs.ecs);
                    try_move_player(-1, 1, &mut gs.ecs);
                }
            }

            // can still skip Turn
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),

            // Might as well accept these too
            VirtualKeyCode::A => return RunState::MoveWeapon,
            VirtualKeyCode::S => return RunState::MoveShield,
            VirtualKeyCode::Z => return RunState::Dodge,

            // Escape dodge mode
            VirtualKeyCode::Escape => return RunState::AwaitingInput,
            _ => return RunState::Dodge,
        },
    }

    RunState::PlayerTurn
}

///
/// SHIELD MOVEMENT SYSTEM
///
pub fn player_shield_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement - shield movement mode
    match ctx.key {
        None => return RunState::Dodge, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(-1, 0, &mut gs.ecs);
                    try_move_player(-1, 0, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(1, 0, &mut gs.ecs);
                    try_move_player(1, 0, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(0, -1, &mut gs.ecs);
                    try_move_player(0, -1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(0, 1, &mut gs.ecs);
                    try_move_player(0, 1, &mut gs.ecs);
                }
            }

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(1, -1, &mut gs.ecs);
                    try_move_player(1, -1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(-1, -1, &mut gs.ecs);
                    try_move_player(-1, -1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(1, 1, &mut gs.ecs);
                    try_move_player(1, 1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(-1, 1, &mut gs.ecs);
                    try_move_player(-1, 1, &mut gs.ecs);
                }
            }

            // can still skip Turn
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),

            // Might as well accept these too
            VirtualKeyCode::A => return RunState::MoveWeapon,
            VirtualKeyCode::S => return RunState::MoveShield,
            VirtualKeyCode::Z => return RunState::Dodge,

            // Escape dodge mode
            VirtualKeyCode::Escape => return RunState::AwaitingInput,
            _ => return RunState::Dodge,
        },
    }

    RunState::PlayerTurn
}

///
/// DODGE SYSTEM
///
pub fn player_dodge_input(gs: &mut State, ctx: &mut Rltk) -> RunState {
    // Player movement - dodge mode
    match ctx.key {
        None => return RunState::Dodge, // Nothing happened
        Some(key) => match key {
            VirtualKeyCode::Left | VirtualKeyCode::Numpad4 | VirtualKeyCode::H => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(-1, 0, &mut gs.ecs);
                    try_move_player(-1, 0, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Right | VirtualKeyCode::Numpad6 | VirtualKeyCode::L => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(1, 0, &mut gs.ecs);
                    try_move_player(1, 0, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Up | VirtualKeyCode::Numpad8 | VirtualKeyCode::K => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(0, -1, &mut gs.ecs);
                    try_move_player(0, -1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Down | VirtualKeyCode::Numpad2 | VirtualKeyCode::J => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(0, 1, &mut gs.ecs);
                    try_move_player(0, 1, &mut gs.ecs);
                }
            }

            // Diagonals
            VirtualKeyCode::Numpad9 | VirtualKeyCode::U => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(1, -1, &mut gs.ecs);
                    try_move_player(1, -1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Numpad7 | VirtualKeyCode::Y => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(-1, -1, &mut gs.ecs);
                    try_move_player(-1, -1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Numpad3 | VirtualKeyCode::N => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(1, 1, &mut gs.ecs);
                    try_move_player(1, 1, &mut gs.ecs);
                }
            }

            VirtualKeyCode::Numpad1 | VirtualKeyCode::B => {
                if player_use_stamina(&mut gs.ecs, 1) {
                    try_move_player(-1, 1, &mut gs.ecs);
                    try_move_player(-1, 1, &mut gs.ecs);
                }
            }

            // can still skip Turn
            VirtualKeyCode::Numpad5 | VirtualKeyCode::Space => return skip_turn(&mut gs.ecs),

            // Might as well accept these too
            VirtualKeyCode::A => return RunState::MoveWeapon,
            VirtualKeyCode::S => return RunState::MoveShield,
            VirtualKeyCode::Z => return RunState::Dodge,

            // Escape dodge mode
            VirtualKeyCode::Escape => return RunState::AwaitingInput,
            _ => return RunState::Dodge,
        },
    }

    RunState::PlayerTurn
}
