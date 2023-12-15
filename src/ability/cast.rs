use std::time::Duration;

use bevy::utils::HashMap;

use crate::{
    ability::Ability,
    actor::player::{
        input::{PlayerInputKeys, PlayerInputQuery},
        reticle::Reticle,
    },
    area::{homing::Homing, AbilityBehavior},
    assets::MaterialPresets,
    camera::OuterGimbal,
    prelude::*,
    stats::{Attributes, HealthMitigatedEvent},
};

use super::{
    bundles::Targetter,
    crowd_control::{CCMap, CCType},
    rank::AbilityRanks,
    DamageType, MaxTargetsHit, TargetsHittable, TargetsInArea, TickBehavior,
};

pub struct CastPlugin;
impl Plugin for CastPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<LogHit>()
            .add_event::<InputCastEvent>()
            .add_event::<AbilityFireEvent>()
            .add_event::<FireHomingEvent>()
            .add_event::<CastEvent>();

        app.add_systems(
            FixedUpdate,
            (
                hover_and_cast,
                cast_ability,
                trigger_cooldown.after(cast_ability),
                tick_cooldowns.after(trigger_cooldown),
                start_ability_windup.after(cast_ability),
                tick_windup_timer,
                place_ability.after(cast_ability),
                place_homing_ability,
            )
                .in_set(InGameSet::Update),
        );
    }
}

// ability selection stuff?

// Make this local only? would be weird to sync other players cast settings, but
// sure?
pub fn hover_and_cast(
    mut query: Query<(
        &mut HoveredAbility,
        &AbilitySlots,
        &AbilityCastSettings,
        PlayerInputQuery,
        Entity,
    )>,
    mut cast_event: EventWriter<InputCastEvent>,
) {
    for (mut hover, ability_slots, cast_settings, input, caster_entity) in &mut query {
        let confirmed = input.just_released(PlayerInputKeys::LEFT_CLICK);
        let rejected = input.just_pressed(PlayerInputKeys::RIGHT_CLICK);
        if rejected {
            continue;
        }

        for (index, ability_key) in input.slots().iter().enumerate() {
            let Some(slot) = Slot::from_index(index) else {
                continue;
            };
            let Some(ability) = ability_slots.get(slot) else {
                continue;
            };
            let hovered = match hover.0 {
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
                    if hovered && confirmed {
                        Action::Cast
                    } else if input.just_pressed(*ability_key) {
                        Action::Hover
                    } else {
                        Action::None
                    }
                }
                CastType::Quick => {
                    if hovered && input.just_released(*ability_key) {
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
                    cast_event.send(InputCastEvent {
                        caster: caster_entity,
                        ability: ability,
                    });
                }
                Action::Hover => {
                    hover.0 = Some(ability.clone());
                }
                _ => {}
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

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct HoveredAbility(pub Option<Ability>);

#[derive(Component, Debug, Default)]
pub struct Casting(pub Option<Ability>);

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

#[derive(Component, Debug, Clone)]
pub struct AbilitySlots {
    abilities: Vec<Ability>,
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

pub fn cast_ability(
    mut players: Query<(&CooldownMap, &CCMap, &mut HoveredAbility)>,
    mut attempt_cast_event: EventReader<InputCastEvent>,
    mut cast_event: EventWriter<CastEvent>,
) {
    for event in attempt_cast_event.read() {
        info!("event: {:?}", event);
        let Ok((cooldowns, ccmap, mut hovered)) = players.get_mut(event.caster) else {
            continue;
        };

        if ccmap.map.contains_key(&CCType::Silence) || ccmap.map.contains_key(&CCType::Stun) {
            continue;
        } // play erro sound for silenced

        if cooldowns.map.contains_key(&event.ability) {
            continue;
        } // play error sound for on CD

        hovered.0 = None;
        cast_event.send(CastEvent {
            caster: event.caster,
            ability: event.ability,
        });
    }
}

#[derive(Event, Debug)]
pub struct InputCastEvent {
    pub caster: Entity,
    pub ability: Ability,
}

#[derive(Event, Debug)]
pub struct CastEvent {
    pub caster: Entity,
    pub ability: Ability,
}

#[derive(Component)]
pub struct WindupTimer(pub Timer);
pub enum CastingStage {
    Charging(Timer),
    Windup(Timer),
    None,
}

fn start_ability_windup(
    mut players: Query<(&mut WindupTimer, &mut Casting)>,
    mut cast_events: EventReader<CastEvent>,
) {
    for event in cast_events.read() {
        let Ok((mut winduptimer, mut casting)) = players.get_mut(event.caster) else {
            continue;
        };
        let windup = event.ability.get_actor_times();
        winduptimer.0 = Timer::new(
            Duration::from_millis((windup * 1000.) as u64),
            TimerMode::Once,
        );
        casting.0 = Some(event.ability);
    }
}

fn tick_windup_timer(
    time: Res<Time>,
    mut players: Query<(Entity, &mut WindupTimer, &mut Casting)>,
    mut fire_event: EventWriter<AbilityFireEvent>,
) {
    for (entity, mut timer, mut casting) in players.iter_mut() {
        let Some(ability) = casting.0 else { continue };
        timer.0.tick(time.delta());
        if timer.0.finished() {
            fire_event.send(AbilityFireEvent {
                caster: entity,
                ability: ability.clone(),
            });
            casting.0 = None;
        }
    }
}

fn trigger_cooldown(
    mut cast_events: EventReader<AbilityFireEvent>,
    mut query: Query<(&mut CooldownMap, &Attributes)>,
) {
    for event in cast_events.read() {
        let Ok((mut cooldowns, attributes)) = query.get_mut(event.caster) else {
            continue;
        };
        let cdr = 1.0 - (attributes.get(Stat::CooldownReduction) / 100.0);

        cooldowns.map.insert(
            event.ability.clone(),
            Timer::new(
                Duration::from_millis((event.ability.get_cooldown() * cdr * 1000.) as u64),
                TimerMode::Once,
            ),
        );
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

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct AbilityMap {
    pub ranks: HashMap<Ability, u32>,
    pub cds: HashMap<Ability, Timer>,
}

#[derive(Component, Reflect, Default, Debug, Clone)]
#[reflect]
pub struct CooldownMap {
    pub map: HashMap<Ability, Timer>,
}

pub struct DamageSum {
    total_change: i32,
    total_mitigated: u32,
    hit_amount: u32,
    sub_list: Vec<HealthMitigatedEvent>,
}

impl DamageSum {
    pub fn add_damage(&mut self, instance: HealthMitigatedEvent) {
        self.total_change += instance.change;
        self.total_mitigated += instance.mitigated;
        self.hit_amount += 1;
        self.sub_list.push(instance);
    }

    pub fn from_instance(instance: HealthMitigatedEvent) -> Self {
        DamageSum {
            total_change: instance.change,
            total_mitigated: instance.mitigated,
            hit_amount: 1,
            sub_list: vec![instance.clone()],
        }
    }

    pub fn total_change(&self) -> i32 {
        self.total_change
    }
    pub fn total_mitigated(&self) -> u32 {
        self.total_mitigated
    }
    pub fn hit_amount(&self) -> u32 {
        self.hit_amount
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum LogSide {
    Incoming,
    Outgoing,
}

#[derive(PartialEq, Eq, Debug)]
pub enum LogType {
    Add,
    Stack,
}

// Change attacker to caster?
#[derive(Event, Debug)]
pub struct LogHit {
    pub sensor: Entity,
    pub attacker: Entity,
    pub defender: Entity,
    pub damage_type: DamageType,
    pub ability: Ability,
    pub change: i32,
    pub mitigated: u32,
    pub log_type: LogType,
    pub log_direction: LogSide,
}

impl LogHit {
    pub fn new(event: HealthMitigatedEvent, log_type: LogType, log_direction: LogSide) -> Self {
        LogHit {
            sensor: event.sensor,
            attacker: event.attacker,
            defender: event.defender,
            damage_type: event.damage_type,
            ability: event.ability,
            change: event.change,
            mitigated: event.mitigated,
            log_type,
            log_direction,
        }
    }
}

#[derive(Component)]
pub struct Caster(pub Entity);

#[derive(Component)]
pub struct Tower;

#[derive(Event)]
pub struct AbilityFireEvent {
    pub caster: Entity,
    pub ability: Ability,
}

#[derive(Event)]
pub struct FireHomingEvent {
    pub caster: Entity,
    pub ability: Ability,
    pub target: Entity,
}

pub fn place_homing_ability(
    mut commands: Commands,
    mut cast_events: EventReader<FireHomingEvent>,
    caster: Query<(&GlobalTransform, &Team)>,
) {
    for event in cast_events.read() {
        let Ok((caster_transform, team)) = caster.get(event.caster) else {
            return;
        };

        let spawned = event
            .ability
            .get_bundle(&mut commands, &caster_transform.compute_transform());

        // Apply general components
        commands.entity(spawned).insert((
            Name::new("Tower shot"),
            team.clone(),
            Homing(event.target),
            Caster(event.caster),
        ));

        // if has a shape
        commands.entity(spawned).insert((
            TargetsInArea::default(),
            TargetsHittable::default(),
            MaxTargetsHit::new(1),
            TickBehavior::individual(),
        ));
    }
}

fn place_ability(
    mut commands: Commands,
    mut cast_events: EventReader<AbilityFireEvent>,
    caster: Query<(&GlobalTransform, &Team, &AbilityRanks)>,
    reticle: Query<&GlobalTransform, With<Reticle>>,
    procmaps: Query<&ProcMap>,
) {
    let Ok(reticle_transform) = reticle.get_single() else {
        return;
    };
    for event in cast_events.read() {
        let Ok((caster_transform, team, _ranks)) = caster.get(event.caster) else {
            return;
        };

        // Get ability-specific components
        let spawned;

        if event.ability.on_reticle() {
            spawned = event
                .ability
                .get_bundle(&mut commands, &reticle_transform.compute_transform());
        } else {
            spawned = event
                .ability
                .get_bundle(&mut commands, &caster_transform.compute_transform());
        }

        // Apply general components
        commands.entity(spawned).insert((
            //Name::new("ability #tick number"),
            team.clone(),
            Caster(event.caster),
        ));

        //let rank = ranks.map.get(&event.ability).cloned().unwrap_or_default();
        //let scaling = rank.current as u32 * event.ability.get_scaling();

        // Apply special proc components
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
