use bevy::utils::HashMap;
use leafwing_input_manager::prelude::*;

use crate::{
    ability::Ability,
    actor::{CooldownMap, InputCastEvent, player::{camera::OuterGimbal, reticle::Reticle, PlayerInput}},
    assets::MaterialPresets,
    prelude::*,
};

use super::bundles::Targetter;

// ability selection stuff?

// Make this local only? would be weird to sync other players cast settings, but
// sure?
pub fn select_ability(
    mut query: Query<(
        &mut HoveredAbility,
        &ActionState<Ability>,
        &AbilityCastSettings,
        Entity,
    )>,
    mut cast_event: EventWriter<InputCastEvent>,
) {
    for (mut hover, ab_state, cast_settings, caster_entity) in &mut query {
        for ability in ab_state.get_just_pressed() {
            let cast_type = cast_settings.0.get(&ability).unwrap_or(&CastType::Normal);
            if *cast_type == CastType::Normal {
                hover.0 = Some(ability.clone());
            } else if *cast_type == CastType::Instant {
                cast_event.send(InputCastEvent {
                    caster: caster_entity,
                    ability: ability,
                });
            }
        }
    }
}

pub fn show_targetter(
    mut commands: Commands,
    query: Query<(&HoveredAbility, &CooldownMap), Changed<HoveredAbility>>,
    reticles: Query<Entity, With<Reticle>>,
    gimbals: Query<Entity, With<OuterGimbal>>,
    targetters: Query<(Entity, &Ability), With<Targetter>>,
    presets: Res<MaterialPresets>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (hovered, cooldowns) in &query {
        let Ok(reticle_entity) = reticles.get_single() else {
            continue;
        };
        let Ok(gimbal_entity) = gimbals.get_single() else {
            continue;
        };
        for (targetter_entity, old_ability) in &targetters {
            if let Some(hovered_ability) = hovered.0 {
                if hovered_ability == *old_ability {
                    continue;
                }
            }
            commands.entity(targetter_entity).despawn_recursive();
        }
        let Some(hovered_ability) = hovered.0 else {
            continue;
        };

        let mut handle = presets
            .0
            .get("blue")
            .unwrap_or(&materials.add(Color::rgb(0.1, 0.2, 0.7).into()))
            .clone();
        if cooldowns.map.contains_key(&hovered_ability) {
            handle = presets
                .0
                .get("white")
                .unwrap_or(&materials.add(Color::rgb(0.4, 0.4, 0.4).into()))
                .clone();
        }
        let targetter = hovered_ability.get_targetter(&mut commands);
        commands
            .entity(targetter)
            .insert((hovered_ability.clone(), handle));

        if hovered_ability.on_reticle() {
            commands.entity(targetter).set_parent(reticle_entity);
        } else {
            commands.entity(targetter).set_parent(gimbal_entity);
        }
    }
}

pub fn change_targetter_color(
    query: Query<(&HoveredAbility, &CooldownMap), Changed<CooldownMap>>,
    mut targetters: Query<(&Ability, &mut Handle<StandardMaterial>), With<Targetter>>,
    presets: Res<MaterialPresets>,
) {
    let Some(castable) = presets.0.get("blue") else {
        return;
    };
    let Some(on_cooldown) = presets.0.get("white") else {
        return;
    };
    for (hovered, cooldowns) in &query {
        let Some(hovered_ability) = hovered.0 else {
            continue;
        };
        let color;
        if cooldowns.map.contains_key(&hovered_ability) {
            color = on_cooldown.clone();
        } else {
            color = castable.clone();
        }
        for (old_ability, mut material) in &mut targetters {
            if old_ability != &hovered_ability {
                continue;
            }
            *material = color.clone();
        }
    }
}

pub fn normal_casting(
    mut query: Query<(&PlayerInput, &mut HoveredAbility, Entity)>,
    mut cast_event: EventWriter<InputCastEvent>,
) {
    for (input, mut hovered, player) in &mut query {
        let Some(hovered_ability) = hovered.0 else {
            continue;
        };
        if input.left_click() {
            cast_event.send(InputCastEvent {
                caster: player,
                ability: hovered_ability,
            });
        }
        if input.right_click() {
            hovered.0 = None;
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct HoveredAbility(pub Option<Ability>);
#[derive(Component, Debug, Default)]
pub struct Casting(pub Option<Ability>);

#[derive(Debug, PartialEq, Eq)]
pub enum CastType {
    Normal,
    Quick,
    Instant,
}

#[derive(Component, Debug)]
pub struct AbilityCastSettings(pub HashMap<Ability, CastType>);

impl Default for AbilityCastSettings {
    fn default() -> Self {
        let settings = HashMap::from([(Ability::BasicAttack, CastType::Instant)]);
        Self(settings)
    }
}
