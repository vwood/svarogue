use super::{
    gamelog::GameLog, EntityMoved, EntryTrigger, InflictsDamage, Map, Name, Position,
    SingleActivation, SufferDamage,
};
use specs::prelude::*;

pub struct TriggerSystem {}

impl<'a> System<'a> for TriggerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Map>,
        WriteStorage<'a, EntityMoved>,
        ReadStorage<'a, Position>,
        ReadStorage<'a, EntryTrigger>,
        ReadStorage<'a, Name>,
        Entities<'a>,
        WriteExpect<'a, GameLog>,
        ReadStorage<'a, InflictsDamage>,
        WriteStorage<'a, SufferDamage>,
        ReadStorage<'a, SingleActivation>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            map,
            mut entity_moved,
            position,
            entry_trigger,
            names,
            entities,
            mut log,
            inflicts_damage,
            mut inflict_damage,
            single_activation,
        ) = data;

        let mut remove_entities: Vec<Entity> = Vec::new();

        for (entity, mut _entity_moved, pos) in (&entities, &mut entity_moved, &position).join() {
            let idx = map.xy_idx(pos.x, pos.y);
            for entity_id in map.tile_content[idx].iter() {
                if entity != *entity_id {
                    let maybe_trigger = entry_trigger.get(*entity_id);
                    match maybe_trigger {
                        None => {}
                        Some(_trigger) => {
                            let name = names.get(*entity_id);
                            if let Some(name) = name {
                                log.entries.push(format!("{} triggers!", &name.name));
                            }

                            let damage = inflicts_damage.get(*entity_id);
                            if let Some(damage) = damage {
                                // particle_builder.request(pos.x, pos.y, rltk::RGB::named(rltk::ORANGE), rltk::RGB::named(rltk::BLACK), rltk::to_cp437('!'), 200.0);
                                SufferDamage::new_damage(
                                    &mut inflict_damage,
                                    entity,
                                    damage.damage,
                                );
                            }

                            let sa = single_activation.get(*entity_id);
                            if let Some(_sa) = sa {
                                remove_entities.push(*entity_id);
                            }
                        }
                    }
                }
            }
        }

        for single_use in remove_entities.iter() {
            entities
                .delete(*single_use)
                .expect("Unable to delete single use item");
        }

        // clear markers
        entity_moved.clear();
    }
}
