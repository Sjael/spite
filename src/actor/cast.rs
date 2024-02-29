use std::{collections::HashMap, time::Duration};

use crate::{
    ability::{Ability, Tags, TargetsHittable, TargetsInArea, Targetter},
    actor::{
        player::input::{PlayerInputKeys, PlayerInputQuery},
        rank::AbilityRanks,
    },
    area::{
        homing::Homing,
        timeline::{AreaTimeline, CastStage},
    },
    assets::MaterialPresets,
    camera::{OuterGimbal, Reticle},
    crowd_control::{CCKind, CCMap},
    prelude::*,
};

pub struct CastPlugin;
impl Plugin for CastPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AbilityFireEvent>();

        app.add_systems(
            FixedUpdate,
            (
                show_targetter,
                change_targetter_color,
                tick_cooldowns,
                tick_casting,
                (hover_and_input, start_casting, place_ability).chain(),
            )
                .in_set(InGameSet::Update),
        );
    }
}

// ability selection stuff?

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
        let Ok(reticle_entity) = reticles.get_single() else { continue };
        let Ok(gimbal_entity) = gimbals.get_single() else { continue };
        for (targetter_entity, old_ability) in &targetters {
            if let Some(hovered_ability) = hovered.0 {
                if hovered_ability == *old_ability {
                    continue
                }
            }
            commands.entity(targetter_entity).despawn_recursive();
        }
        let Some(hovered_ability) = hovered.0 else { continue };

        let mut handle = presets
            .0
            .get("blue")
            .unwrap_or(&materials.add(Color::rgb(0.1, 0.2, 0.7)))
            .clone();
        if cooldowns.map.contains_key(&hovered_ability) {
            handle = presets
                .0
                .get("white")
                .unwrap_or(&materials.add(Color::rgb(0.4, 0.4, 0.4)))
                .clone();
        }
        let targetter = commands.spawn(hovered_ability.hover()).insert(handle).id();

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
    let Some(castable) = presets.0.get("blue") else { return };
    let Some(on_cooldown) = presets.0.get("white") else { return };
    for (hovered, cooldowns) in &query {
        let Some(hovered_ability) = hovered.0 else { continue };
        let color;
        if cooldowns.map.contains_key(&hovered_ability) {
            color = on_cooldown.clone();
        } else {
            color = castable.clone();
        }
        for (old_ability, mut material) in &mut targetters {
            if old_ability != &hovered_ability {
                continue
            }
            *material = color.clone();
        }
    }
}

// Make this local only? would be weird to sync other players cast settings, but
// sure?
fn hover_and_input(
    mut query: Query<(
        &mut HoveredAbility,
        &AbilitySlots,
        &AbilityCastSettings,
        &mut Casting,
        PlayerInputQuery,
    )>,
) {
    for (mut hover, ability_slots, cast_settings, mut casting, input) in &mut query {
        let confirmed = input.just_released(PlayerInputKeys::LEFT_CLICK);
        let rejected = input.just_pressed(PlayerInputKeys::RIGHT_CLICK);
        if rejected {
            hover.0 = None;
            continue
        }

        for (index, ability_key) in input.slots().iter().enumerate() {
            let Some(slot) = Slot::from_index(index) else { continue };
            let Some(ability) = ability_slots.get(slot) else { continue };
            let is_hovered = match hover.0 {
                Some(hovered) if hovered == ability => true,
                _ => false,
            };

            enum Action {
                None,
                Hover,
                Cast,
            }

            let cast_type = cast_settings.0.get(&ability).unwrap_or(&CastType::Normal);
            let action = match cast_type {
                CastType::Normal => {
                    if is_hovered && confirmed {
                        Action::Cast
                    } else if input.just_pressed(*ability_key) {
                        Action::Hover
                    } else {
                        Action::None
                    }
                }
                CastType::Quick => {
                    if is_hovered && input.just_released(*ability_key) {
                        Action::Cast
                    } else if input.just_pressed(*ability_key) {
                        Action::Hover
                    } else {
                        Action::None
                    }
                }
                CastType::Instant => {
                    if input.just_pressed(*ability_key) {
                        Action::Cast
                    } else {
                        Action::None
                    }
                }
            };

            match action {
                Action::Cast => {
                    casting.next.push(ability);
                    hover.0 = None;
                }
                Action::Hover => {
                    hover.0 = Some(ability.clone());
                }
                _ => {}
            }
        }
    }
}

fn start_casting(mut actors: Query<(&CCMap, &mut Casting), Changed<Casting>>) {
    for (cc, mut casting) in actors.iter_mut() {
        if cc.map.contains_key(&CCKind::Silence) || cc.map.contains_key(&CCKind::Stun) {
            continue
        } // play error sound for silenced
        for ability in casting.next.clone() {
            if casting.current.contains_key(&ability) {
                continue
            }
            casting.current.insert(ability, ability.get_area_timeline());
        }
        casting.next = Vec::new();
    }
}

fn tick_casting(
    time: Res<Time>,
    mut casters: Query<(&mut Casting, &mut Attributes, &mut CooldownMap, Entity)>,
    mut cast_events: EventWriter<AbilityFireEvent>,
) {
    for (mut casting, mut attributes, mut cooldowns, entity) in casters.iter_mut() {
        casting.current.retain(|ability, timeline| {
            timeline.tick(time.delta());
            if timeline.stage == CastStage::Casted {
                // have this check in here so you can predict when an ability will be up for skill cap
                if cooldowns.map.contains_key(&ability) {
                    return false
                } // play error sound for on CD

                let cdr = 1.0 - (attributes.get(Stat::CooldownReduction) / 100.0);

                let resource = attributes.get_mut(Stat::CharacterResource);
                if *resource < ability.get_cost() as f32 {
                    // if not enough resource
                    // Recoil damage and continue
                    let hp = attributes.get_mut(Stat::Health);
                    *hp -= *hp * 0.1;
                    return false
                }
                cast_events.send(AbilityFireEvent {
                    caster: entity,
                    ability: ability.clone(),
                    extras: Vec::new(),
                });
                *resource -= ability.get_cost() as f32;
                cooldowns.map.insert(
                    ability.clone(),
                    Timer::new(
                        Duration::from_millis((ability.get_cooldown() * cdr * 1000.) as u64),
                        TimerMode::Once,
                    ),
                );
                return false
            }
            true
        });
    }
}

// Move these to character file, since mobs will be cc'd and buffed/cooldowns
// too AND MAKE GENERIC ⬇️⬇️⬇️

fn tick_cooldowns(
    time: Res<Time>,
    mut query: Query<&mut CooldownMap>,
    //mut cd_events: EventWriter<CooldownFreeEvent>,
) {
    for mut cooldowns in &mut query {
        // remove if finished
        cooldowns.map.retain(|_, timer| {
            timer.tick(time.delta());
            !timer.finished()
        });
    }
}

fn place_ability(
    mut commands: Commands,
    mut cast_events: EventReader<AbilityFireEvent>,
    caster: Query<(&GlobalTransform, &Team, &AbilityRanks)>,
    reticle: Query<&GlobalTransform, With<Reticle>>,
    procmaps: Query<&ProcMap>,
) {
    let Ok(reticle_transform) = reticle.get_single() else { return };
    for event in cast_events.read() {
        let Ok((caster_transform, team, _ranks)) = caster.get(event.caster) else { return };
        let ability = event.ability;
        // Get ability-specific components
        let transform = if event.ability.on_reticle() {
            reticle_transform.compute_transform()
        } else {
            caster_transform.compute_transform()
        };

        // TODO if ability actually spawns something, which is going to be like 80% of the time
        // other cases include Zeus Detonate, self buffs, mobility
        let spawned = commands
            .spawn((
                Name::new(ability.get_name()),
                ability,
                ability.get_shape(),
                SpatialBundle::from_transform(transform.clone()),
                Sensor,
                RigidBody::Kinematic,
                // Apply team and caster components for figuring out damage
                team.clone(),
                Caster(event.caster),
                AreaTimeline::new_at_stage(ability.get_timeline_blueprint(), CastStage::Windup),
                ability.get_damage_type(),
                TargetsHittable::default(),
                TargetsInArea::default(),
            ))
            .id();

        if ability.get_speed() > 1.0 {
            let direction = transform.rotation * -Vec3::Z;
            commands
                .entity(spawned)
                .insert(LinearVelocity(direction * ability.get_speed()));
        }

        //let rank = ranks.map.get(&event.ability).cloned().unwrap_or_default();
        //let scaling = rank.current as u32 * event.ability.get_scaling();

        // TODO Scale these tags with ranks appropriately
        commands.entity(spawned).insert(Tags(ability.get_tags()));

        // like MaxTargets before despawn, Ticks, etc. Rework later somehow
        ability.add_unique_components(&mut commands, spawned);

        for extra in event.extras.iter() {
            match extra {
                AbilityExtras::Homing(target) => {
                    commands.entity(spawned).insert(Homing(*target));
                }
            }
        }

        // Apply special procs from the caster's proc list component (qin sais, exe, etc)
        if let Ok(procmap) = procmaps.get(event.caster) {
            if let Some(behaviors) = procmap.0.get(&event.ability) {
                for behavior in behaviors {
                    match behavior {
                        AbilityBehavior::Homing => (),
                        AbilityBehavior::OnHit => (),
                    }
                }
            }
        }
    }
}

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct HoveredAbility(pub Option<Ability>);

#[derive(Component, Debug, Default)]
pub struct Casting {
    pub current: HashMap<Ability, AreaTimeline>,
    pub next: Vec<Ability>,
}

#[derive(Event)]
pub struct AbilityFireEvent {
    pub caster: Entity,
    pub ability: Ability,
    pub extras: Vec<AbilityExtras>,
}

pub enum AbilityExtras {
    Homing(Entity),
    // idk cant think of any others rn
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct CooldownMap {
    pub map: HashMap<Ability, Timer>,
}

#[derive(Component)]
pub struct Caster(pub Entity);

#[derive(Component)]
pub struct Tower;

/// Modes of inputting abilities casts.
#[derive(Debug, PartialEq, Eq)]
pub enum CastType {
    /// Press and release ability slot to select ability,
    /// then confirm cast with left click (primary action).
    Normal,
    /// Press and release ability slot to cast.
    Quick,
    /// Press ability slot to cast.
    Instant,
}

#[derive(Copy, Clone)]
pub enum Slot {
    Primary = 0,
    Slot1 = 1,
    Slot2 = 2,
    Slot3 = 3,
    Slot4 = 4,
}

impl Slot {
    pub fn from_index(index: usize) -> Option<Slot> {
        match index {
            0 => Some(Slot::Primary),
            1 => Some(Slot::Slot1),
            2 => Some(Slot::Slot2),
            3 => Some(Slot::Slot3),
            4 => Some(Slot::Slot4),
            _ => None,
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct AbilitySlots {
    abilities: Vec<Ability>,
}

impl AbilitySlots {
    pub fn new() -> Self {
        Self {
            abilities: Vec::with_capacity(6),
        }
    }

    pub fn set(&mut self, slot: Slot, ability: Ability) {
        self.abilities.insert(slot as usize, ability);
    }

    pub fn with(mut self, slot: Slot, ability: Ability) -> Self {
        self.set(slot, ability);
        self
    }

    pub fn get(&self, slot: Slot) -> Option<Ability> {
        self.abilities.get(slot as usize).copied()
    }

    pub fn abilities(&self) -> impl Iterator<Item = Ability> + '_ {
        self.abilities.iter().copied()
    }
}

#[derive(Component, Debug)]
pub struct AbilityCastSettings(pub HashMap<Ability, CastType>);

impl Default for AbilityCastSettings {
    fn default() -> Self {
        let settings = HashMap::from([(Ability::BasicAttack, CastType::Instant)]);
        Self(settings)
    }
}
