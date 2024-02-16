use bevy::prelude::*;

use crate::{
    actor::KillEvent,
    prelude::ActorType,
    stats::{Attributes, Stat},
};

pub struct HunterPlugin;

impl Plugin for HunterPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, gen_instinct);
    }
}

fn gen_instinct(
    mut kill_events: EventReader<KillEvent>,
    mut query: Query<&mut Attributes, With<Hunter>>,
    damned: Query<&ActorType>,
) {
    for event in kill_events.read() {
        let Ok(mut attrs) = query.get_mut(event.killer) else { continue };
        let Ok(actor_type) = damned.get(event.damned) else { continue };
        let max = attrs.get(Stat::CharacterResourceMax);
        let resource = attrs.get_mut(Stat::CharacterResource);

        if let ActorType::Player(_) = actor_type {
            *resource = max;
        } else {
            *resource += 1.0;
        }
        *resource = resource.clamp(0.0, max);
    }
}

#[derive(Component)]
pub struct Hunter;
