//! Gathering of entities that should be considered in filtering for this abillity.

use std::time::Duration;

use bevy::utils::HashSet;

use crate::prelude::*;


#[derive(Component, Default)]
pub struct Collected(Vec<Entity>);

impl Collected {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, entity: Entity) {
        self.0.push(entity);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

pub fn clear_collected(mut collectors: Query<&mut Collected>) {
    for mut collected in &mut collectors {
        collected.clear();
    }
}

// Filters for collected entities.
//
// This includes stuff like de-duplicating if the entity was already hit by this ability.

/// Filter for entities already hit by this ability.
#[derive(Component, Default)]
pub struct AlreadyHit(HashSet<Entity>);

pub fn filter_already_hit(mut filtering: Query<(&mut Collected, &AlreadyHit)>) {
    for (mut collected, already_hit) in &mut filtering {
        collected.0.retain(|entity| !already_hit.0.contains(entity));
    }
}

/// Add entities that pass all filters to the `AlreadyHit` components.
pub fn update_already_hit(mut hitting: Query<(&mut AlreadyHit, &Collected)>) {
    for (mut already_hit, collected) in &mut hitting {
        for entity in &collected.0 {
            already_hit.0.insert(*entity);
        }
    }
}

/// Filter entities that have been hit within a period of time in the past.
#[derive(Component, Default)]
pub struct TimedHit {
    /// Entities current hit.
    pub tracking: Vec<(Entity, Timer)>,
    /// Duration in seconds before this ability can be re-applied.
    pub duration: f32,
}

impl TimedHit {
    pub fn contains(&self, entity: Entity) -> bool {
        self.tracking.iter().any(|(hit, _)| *hit == entity)
    }

    pub fn insert(&mut self, entity: Entity) {
        self.tracking.push((
            entity,
            Timer::new(Duration::from_secs_f32(self.duration), TimerMode::Once),
        ));
    }

    pub fn advance_by(&mut self, duration: Duration) {
        for (_, ref mut timer) in &mut self.tracking {
            timer.tick(duration);
        }

        self.clean();
    }

    pub fn clean(&mut self) {
        self.tracking.retain(|(_, timer)| !timer.finished());
    }
}

pub fn filter_timed_hit(mut filtering: Query<(&mut Collected, &TimedHit)>) {
    for (mut collected, timed_hit) in &mut filtering {
        collected.0.retain(|entity| !timed_hit.contains(*entity));
    }
}

pub fn update_timed_hit(time: Res<Time>, mut hitting: Query<(&mut TimedHit, &Collected)>) {
    for (mut timed_hit, collected) in &mut hitting {
        for entity in &collected.0 {
            timed_hit.insert(*entity);
        }

        timed_hit.advance_by(time.delta());
    }
}
