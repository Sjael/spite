use bevy::prelude::*;

// Make only increment when damageable? Aegis before guarenteed death is meta
// now LMAO
pub(super) fn increment_bounty(mut the_notorious: Query<&mut Bounty>, time: Res<Time>) {
    for mut wanted in the_notorious.iter_mut() {
        wanted.gold += 2.0 * time.delta_seconds();
        wanted.xp += 4.0 * time.delta_seconds();
    }
}

#[derive(Component, Debug, Clone, Reflect)]
#[reflect(Component)]
pub struct Bounty {
    pub xp: f32,
    pub gold: f32,
}

impl Default for Bounty {
    fn default() -> Self {
        Self { xp: 200.0, gold: 250.0 }
    }
}
