use bevy::prelude::*;

use crate::{
    area::queue::HealthChangeEvent,
    stats::{Attributes, Stat},
};

pub struct WarriorPlugin;

impl Plugin for WarriorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, gen_fury);
    }
}

pub fn gen_fury(mut damage_events: EventReader<HealthChangeEvent>, mut query: Query<&mut Attributes, With<Warrior>>) {
    for event in damage_events.read() {
        let mut attrs = if let Ok(attrs) = query.get_mut(event.attacker) {
            attrs
        } else if let Ok(attrs) = query.get_mut(event.defender) {
            attrs
        } else {
            continue
        };
        let max = attrs.get(Stat::CharacterResourceMax);
        let resource = attrs.get_mut(Stat::CharacterResource);
        *resource += 1.0;
        *resource = resource.clamp(0.0, max);
    }
}

#[derive(Component)]
pub struct Warrior;
