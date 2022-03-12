use super::{
    map::find_empty_adjacent, map::MAPWIDTH, random_table::RandomTable, AreaOfEffect, Attribute,
    Attributes, BlocksTile, CombatStats, Confusion, Consumable, DefenseBonus, EntryTrigger,
    EquipmentSlot, Equippable, InflictsDamage, Item, MeleePowerBonus, Monster, Name, Player,
    Position, ProvidesHealing, Ranged, Rect, Renderable, SerializeMe, SingleActivation, Viewshed,
    WeaponStats,
};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

/// Spawns the player and returns their entity object.
pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    let entity = ecs
        .create_entity()
        .with(Position { x: player_x, y: player_y })
        .with(Renderable {
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(Player {})
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Name { name: "Player".to_string() })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            stamina: 10,
            max_stamina: 10,
            defense: 2,
            power: 5,
        })
        .with(Attributes {
            strength: Attribute { base: 10, modifiers: 0, bonus: 0 },
            dexterity: Attribute { base: 10, modifiers: 0, bonus: 0 },
            endurance: Attribute { base: 10, modifiers: 0, bonus: 0 },
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();

    let (x, y) = find_empty_adjacent(ecs, player_x, player_y);
    weapon_entity(ecs, x, y, entity);

    entity
}

/// Spawns the weapon and returns the entity object.
pub fn weapon_entity(ecs: &mut World, x: i32, y: i32, owner: Entity) -> Entity {
    ecs.create_entity()
        .with(Position { x: x, y: y })
        .with(Renderable {
            glyph: rltk::to_cp437('*'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 0,
        })
        .with(WeaponStats { power: 8, owner: owner })
        .with(CombatStats {
            max_hp: 30,
            hp: 30,
            stamina: 1,
            max_stamina: 1,
            defense: 20,
            power: 8,
        })
        .with(Name { name: "Player Weapon".to_string() })
        .marked::<SimpleMarker<SerializeMe>>()
        .build()
}

/// Resets the weapon location
pub fn reset_weapon_locations(ecs: &mut World) {
    let mut weapon_stats = ecs.write_storage::<WeaponStats>();
    let players = ecs.read_storage::<Player>();
    let mut positions = ecs.write_storage::<Position>();
    let entities = ecs.entities();

    for (entity, stats) in (&entities, &weapon_stats).join() {
        let (x, y);
        {
            let position = positions.get(stats.owner).unwrap();

            // silly unstable destructuring assignments
            let new_xy = find_empty_adjacent(ecs, position.x, position.y);
            x = new_xy.0;
            y = new_xy.1;
        }

        let pos = positions.get_mut(entity).unwrap();
        pos.x = x;
        pos.y = y;
    }
}

const MAX_MONSTERS: i32 = 7;

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Troglodyte", 10)
        .add("Skeleton", 2 + map_depth)
        .add("Ogre", 1 + map_depth)
        .add("Health Potion", 7)
        // .add("Fireball Scroll", 2 + map_depth)
        // .add("Confusion Scroll", 2 + map_depth)
        // .add("Magic Missile Scroll", 4)
        .add("Dagger", 3 * 2)
        .add("Shield", 3)
        .add("Longsword", map_depth * 2)
        .add("Halberd", map_depth)
        .add("Tower Shield", map_depth)
        .add("Bear Trap", 5)
}

/// Fills a room with stuff!
#[allow(clippy::map_entry)]
pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(2, MAX_MONSTERS) + (map_depth - 2);

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAPWIDTH) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the monsters
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;
        spawn_item(ecs, x, y, spawn.1);
    }
}

fn spawn_item(ecs: &mut World, x: i32, y: i32, item: &str) {
    match item.as_ref() {
        "Troglodyte" => troglodyte(ecs, x, y),
        "Skeleton" => skeleton(ecs, x, y),
        "Ogre" => ogre(ecs, x, y),
        "Health Potion" => health_potion(ecs, x, y),
        "Fireball Scroll" => fireball_scroll(ecs, x, y),
        "Confusion Scroll" => confusion_scroll(ecs, x, y),
        "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
        "Dagger" => dagger(ecs, x, y),
        "Shield" => shield(ecs, x, y),
        "Longsword" => longsword(ecs, x, y),
        "Halberd" => halberd(ecs, x, y),
        "Tower Shield" => tower_shield(ecs, x, y),
        "Bear Trap" => bear_trap(ecs, x, y),
        _ => {}
    }
}

pub fn spawn_locations(ecs: &mut World, positions: &[Position], map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();

        for position in positions {
            let idx = (position.y as usize * MAPWIDTH) + position.x as usize;
            spawn_points.insert(idx, spawn_table.roll(&mut rng));
        }
    }

    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAPWIDTH) as i32;
        let y = (*spawn.0 / MAPWIDTH) as i32;
        spawn_item(ecs, x, y, spawn.1);
    }
}

fn skeleton(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('S'), "Skeleton", RGB::named(rltk::WHITE), 8, 5, 2);
}
fn troglodyte(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('T'), "Troglodyte", RGB::named(rltk::BROWN1), 6, 4, 1);
}
fn ogre(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('O'), "Ogre", RGB::named(rltk::GREEN), 12, 6, 3);
}

fn monster<S: ToString>(
    ecs: &mut World,
    x: i32,
    y: i32,
    glyph: rltk::FontCharType,
    name: S,
    fg: RGB,
    hp: i32,
    power: i32,
    defense: i32,
) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable { glyph, fg: fg, bg: RGB::named(rltk::BLACK), render_order: 1 })
        .with(Viewshed { visible_tiles: Vec::new(), range: 8, dirty: true })
        .with(Monster {})
        .with(Name { name: name.to_string() })
        .with(BlocksTile {})
        .with(CombatStats {
            max_hp: hp,
            hp: hp,
            stamina: 2,
            max_stamina: 2,
            defense: defense,
            power: power,
        })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('!'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Health Potion".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(ProvidesHealing { heal_amount: 8 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Magic Missile Scroll".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Fireball Scroll".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(InflictsDamage { damage: 20 })
        .with(AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Confusion Scroll".to_string() })
        .with(Item {})
        .with(Consumable {})
        .with(Ranged { range: 6 })
        .with(Confusion { turns: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Dagger".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus { power: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Shield".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Shield })
        .with(DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn longsword(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Longsword".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus { power: 4 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn halberd(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::WHITE),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Halberd".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Melee })
        .with(MeleePowerBonus { power: 6 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn tower_shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Tower Shield".to_string() })
        .with(Item {})
        .with(Equippable { slot: EquipmentSlot::Shield })
        .with(DefenseBonus { defense: 2 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}

fn bear_trap(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(Position { x, y })
        .with(Renderable {
            glyph: rltk::to_cp437('^'),
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
            render_order: 2,
        })
        .with(Name { name: "Bear Trap".to_string() })
        .with(EntryTrigger {})
        .with(SingleActivation {})
        .with(InflictsDamage { damage: 6 })
        .marked::<SimpleMarker<SerializeMe>>()
        .build();
}
