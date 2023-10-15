use bevy::{prelude::*, ui::RelativeCursorPosition};
use bevy_tweening::TweenCompleted;

use crate::{
    ability::Ability,
    actor::{
        buff::{BuffAddEvent, BuffStackEvent, BuffType},
        crowd_control::{CCMap, CCType},
        player::Player,
        stats::*,
        view::Spectating,
        CastEvent, CooldownMap, LogHit, LogSide, LogType, Tower, WindupTimer,
    },
    assets::{Fonts, Icons, Images},
    game_manager::{AbilityFireEvent, Team},
    input::SlotAbilityMap,
    ui::ui_bundles::*,
};

pub fn add_player_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<RootUI>>,
    player_query: Query<&SlotAbilityMap, Added<Player>>,
    fonts: Res<Fonts>,
    icons: Res<Icons>,
) {
    let Ok(root_ui) = ui_query.get_single() else { return };
    for ability_map in player_query.iter() {
        commands.entity(root_ui).with_children(|parent| {
            parent.spawn(character_ui()).with_children(|parent| {
                // Bottom Container
                parent
                    .spawn(player_bottom_container())
                    .with_children(|parent| {
                        // Buffs / Debuffs
                        parent.spawn(effect_bar()).with_children(|parent| {
                            parent.spawn(buff_bar());
                            parent.spawn(debuff_bar());
                        });
                        // Resource Bars
                        parent.spawn(player_bars_wrapper()).with_children(|parent| {
                            parent.spawn(bar_background(20.0)).with_children(|parent| {
                                parent
                                    .spawn(bar_fill(Color::rgb(0.27, 0.77, 0.26)))
                                    .insert(HealthBarUI);
                                parent.spawn(bar_text_wrapper()).with_children(|parent| {
                                    parent
                                        .spawn(custom_text(&fonts, 18.0, -1.0))
                                        .insert(HealthBarText);
                                });
                            });
                            parent.spawn(bar_background(14.0)).with_children(|parent| {
                                parent
                                    .spawn(bar_fill(Color::rgb(0.92, 0.24, 0.01)))
                                    .insert(ResourceBarUI);
                                parent.spawn(bar_text_wrapper()).with_children(|parent| {
                                    parent
                                        .spawn(custom_text(&fonts, 14.0, -2.0))
                                        .insert(ResourceBarText);
                                });
                            });
                        });
                        // CDs
                        parent.spawn((ability_holder(), ability_map.clone()));
                    });
                // CC on self
                parent.spawn(cc_holder()).with_children(|parent| {
                    parent.spawn(cc_holder_top()).with_children(|parent| {
                        parent
                            .spawn(cc_icon(CCType::Root, &icons))
                            .insert(CCIconSelf);
                        parent.spawn(plain_text("", 24, &fonts)).insert(CCSelfLabel);
                    });
                    parent.spawn(bar_background(6.0)).with_children(|parent| {
                        parent
                            .spawn(bar_fill(Color::WHITE.with_a(0.9)))
                            .insert(CCBarSelfFill);
                    });
                });
                // castbar
                parent.spawn(cast_bar_holder()).with_children(|parent| {
                    //parent.spawn(cc_icon(CCType::Root, &icons,)).insert(CCIconSelf);
                    parent.spawn(bar_background(2.0)).with_children(|parent| {
                        parent
                            .spawn(bar_fill(Color::YELLOW.with_a(0.9)))
                            .insert(CastBarFill);
                    });
                });
                // objective health
                parent
                    .spawn(objective_health_bar_holder())
                    .with_children(|parent| {
                        parent
                            .spawn(plain_text("", 18, &fonts))
                            .insert(ObjectiveName);
                        parent.spawn(bar_background(24.0)).with_children(|parent| {
                            parent
                                .spawn(bar_fill(Color::rgba(1.0, 0.2, 0.2, 0.9)))
                                .insert(ObjectiveHealthFill);
                        });
                    });
            });
        });
    }
}

pub fn add_ability_icons(
    mut commands: Commands,
    query: Query<(Entity, &SlotAbilityMap), (With<AbilityHolder>, Changed<SlotAbilityMap>)>, // Changed<AbilityHolder> for changing spells midgame
    icons: Res<Icons>,
    fonts: Res<Fonts>,
) {
    for (entity, ability_map) in query.iter() {
        for (_, ability) in &ability_map.map {
            let ability_icon = commands
                .spawn((
                    ability_image(ability.get_image(&icons)),
                    ability.get_tooltip(),
                    Hoverable,
                    RelativeCursorPosition::default(),
                ))
                .id();

            let cd_text = commands.spawn((cd_text(&fonts), ability.clone())).id();

            commands.entity(cd_text).set_parent(ability_icon);
            commands.entity(ability_icon).set_parent(entity);
        }
    }
}

#[derive(Component)]
pub struct BarTrack {
    pub entity: Entity,
    pub current: AttributeTag,
    pub max: AttributeTag,
}

pub fn bar_track(
    query: Query<&Attributes, Changed<Attributes>>,
    mut bar_query: Query<(&mut Style, &BarTrack)>,
) {
    for (mut style, tracking) in &mut bar_query {
        let Ok(attributes) = query.get(tracking.entity) else { continue };
        let current = *attributes.get(&tracking.current).unwrap_or(&0.0);
        let max = *attributes.get(&tracking.max).unwrap_or(&100.0);
        let new_size = current / max;
        style.width = Val::Percent(new_size * 100.0);
    }
}

pub fn update_cc_bar(
    spectating: Res<Spectating>,
    cc_maps: Query<&CCMap>,
    mut cc_bar_fill: Query<&mut Style, With<CCBarSelfFill>>,
) {
    let Ok(cc_of_spectating) = cc_maps.get(spectating.0) else { return };
    let cc_vec = Vec::from_iter(cc_of_spectating.map.clone());
    let Some((_, cc_timer)) = cc_vec.get(0) else { return };
    let Ok(mut bar) = cc_bar_fill.get_single_mut() else { return };
    bar.width = Val::Percent(cc_timer.percent_left() * 100.0);
}

pub fn update_health(
    query: Query<&Attributes, (With<Player>, Changed<Attributes>)>,
    mut text_query: Query<&mut Text, With<HealthBarText>>,
    mut bar_query: Query<&mut Style, With<HealthBarUI>>,
    fonts: Res<Fonts>,
    spectating: Res<Spectating>,
) {
    let Ok(mut text) = text_query.get_single_mut() else { return };
    let Ok(mut bar) = bar_query.get_single_mut() else { return };
    let Ok(attributes) = query.get(spectating.0) else { return };
    let current = *attributes.get(&Stat::Health.as_tag()).unwrap_or(&0.0);
    let regen = *attributes.get(&Stat::HealthRegen.as_tag()).unwrap_or(&0.0);
    let max = *attributes.get(&Stat::HealthMax.as_tag()).unwrap_or(&100.0);

    let current_text = format!("{}", current.trunc());
    let max_text = format!(" / {}", max.trunc());
    let regen_text = format!(" (+{})", regen.trunc());
    *text = Text::from_sections([
        TextSection {
            value: current_text,
            style: TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: 18.0,
                color: Color::WHITE,
            },
        },
        TextSection {
            value: max_text,
            style: TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: 18.0,
                color: Color::YELLOW,
            },
        },
        TextSection {
            value: regen_text,
            style: TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: 18.0,
                color: Color::WHITE,
            },
        },
    ]);
    let new_size = current / max;
    bar.width = Val::Percent(new_size * 100.0);
}

pub fn update_character_resource(
    query: Query<&Attributes, (With<Player>, Changed<Attributes>)>,
    mut text_query: Query<&mut Text, With<ResourceBarText>>,
    mut bar_query: Query<&mut Style, With<ResourceBarUI>>,
    spectating: Res<Spectating>,
) {
    let Ok(mut text) = text_query.get_single_mut() else { return };
    let Ok(mut bar) = bar_query.get_single_mut() else { return };
    let Ok(attributes) = query.get(spectating.0) else { return };
    let current = *attributes
        .get(&Stat::CharacterResource.as_tag())
        .unwrap_or(&0.0);
    let regen = *attributes
        .get(&Stat::CharacterResourceRegen.as_tag())
        .unwrap_or(&0.0);
    let max = *attributes
        .get(&Stat::CharacterResourceMax.as_tag())
        .unwrap_or(&100.0);

    text.sections[0].value = format!("{} / {} (+{})", current.trunc(), max.trunc(), regen.trunc());

    let new_size = current / max;
    bar.width = Val::Percent(new_size * 100.0);
}

pub fn update_objective_health(
    focused_health_entity: Res<FocusedHealthEntity>,
    mut bar_query: Query<&mut Style, With<ObjectiveHealthFill>>,
    query: Query<&Attributes, Changed<Attributes>>,
) {
    let Some(focused_entity) = focused_health_entity.0 else { return };
    let Ok(mut bar) = bar_query.get_single_mut() else { return };
    let Ok(attributes) = query.get(focused_entity) else { return };
    let current = *attributes.get(&Stat::Health.as_tag()).unwrap_or(&0.0);
    let max = *attributes.get(&Stat::HealthMax.as_tag()).unwrap_or(&100.0);

    let new_size = current / max;
    bar.width = Val::Percent(new_size * 100.0);
}

pub fn update_gold_inhand(
    query: Query<&Attributes, (With<Player>, Changed<Attributes>)>,
    mut text_query: Query<&mut Text, With<GoldInhand>>,
    spectating: Res<Spectating>,
) {
    let Ok(attributes) = query.get(spectating.0) else { return };
    let gold = *attributes.get(&Stat::Gold.as_tag()).unwrap_or(&0.0);
    for mut text in text_query.iter_mut() {
        text.sections[0].value = gold.trunc().to_string();
    }
}

pub fn toggle_cc_bar(
    spectating: Res<Spectating>,
    cc_maps: Query<&CCMap, Changed<CCMap>>,
    mut cc_bar: Query<&mut Visibility, With<CCSelf>>,
    mut cc_icon: Query<&mut UiImage, With<CCIconSelf>>,
    mut cc_text: Query<&mut Text, With<CCSelfLabel>>,
    icons: Res<Icons>,
) {
    let Ok(cc_of_spectating) = cc_maps.get(spectating.0) else { return };
    let Ok(mut vis) = cc_bar.get_single_mut() else { return };
    let Ok(mut image) = cc_icon.get_single_mut() else { return };
    let Ok(mut text) = cc_text.get_single_mut() else { return };
    if cc_of_spectating.map.is_empty() {
        *vis = Visibility::Hidden;
    } else {
        let cc_vec = Vec::from_iter(cc_of_spectating.map.clone());
        let Some((top_cc, _)) = cc_vec.get(0) else { return };
        image.texture = top_cc.clone().get_icon(&icons);
        text.sections[0].value = top_cc.to_text();
        *vis = Visibility::Visible;
    }
}

pub fn update_cast_bar(
    spectating: Res<Spectating>,
    windup_query: Query<&WindupTimer>,
    mut cast_bar_fill: Query<&mut Style, With<CastBarFill>>,
) {
    let Ok(windup) = windup_query.get(spectating.0) else { return };
    let Ok(mut style) = cast_bar_fill.get_single_mut() else { return };
    style.width = Val::Percent(windup.0.percent() * 100.0);
}

pub fn toggle_cast_bar(
    spectating: Res<Spectating>,
    mut bar: Query<&mut Visibility, With<CastBar>>,
    mut cast_events: EventReader<CastEvent>,
    mut fire_events: EventReader<AbilityFireEvent>,
) {
    let Ok(mut vis) = bar.get_single_mut() else { return };
    for event in cast_events.iter() {
        if event.caster != spectating.0 {
            continue
        }
        *vis = Visibility::Visible;
    }
    for event in fire_events.iter() {
        if event.caster != spectating.0 {
            continue
        }
        *vis = Visibility::Hidden;
    }
}

pub fn update_cooldowns(
    spectating: Res<Spectating>,
    cooldown_query: Query<&CooldownMap>,
    cooldown_changed_query: Query<&CooldownMap, Changed<CooldownMap>>,
    mut text_query: Query<(&mut Text, &Ability, &Parent), With<CooldownIconText>>,
    mut image_query: Query<&mut BackgroundColor, With<UiImage>>,
) {
    // tick existing cooldowns
    let Ok(cooldowns) = cooldown_query.get(spectating.0) else { return };
    for (mut text, ability, _) in text_query.iter_mut() {
        if cooldowns.map.contains_key(ability) {
            let Some(timer) = cooldowns.map.get(ability) else { continue };
            let newcd = timer.remaining_secs() as u32;
            text.sections[0].value = newcd.to_string();
        }
    }
    // set bg color only when cooldowns change
    if let Ok(cooldowns_changed) = cooldown_changed_query.get(spectating.0) {
        for (mut text, ability, parent) in text_query.iter_mut() {
            let Ok(mut background_color) = image_query.get_mut(parent.get()) else { continue };

            if cooldowns_changed.map.contains_key(ability) {
                *background_color = Color::rgb(0.2, 0.2, 0.2).into();
            } else {
                text.sections[0].value = String::from("");
                *background_color = Color::WHITE.into();
            }
        }
    }
}

pub fn update_buff_timers(
    mut text_query: Query<(&mut Text, &Parent), With<BuffDurationText>>,
    timer_query: Query<&DespawnTimer>,
) {
    for (mut text, parent) in text_query.iter_mut() {
        let Ok(despawn_timer) = timer_query.get(parent.get()) else { continue };
        let remaining = despawn_timer.0.remaining_secs() as u32;
        text.sections[0].value = remaining.to_string();
    }
}

pub fn update_buff_stacks(
    mut stack_events: EventReader<BuffStackEvent>,
    spectating: Res<Spectating>,
    mut buff_holders: Query<(Entity, &BuffId, &mut DespawnTimer)>,
    children_query: Query<&Children>,
    mut stacks: Query<(&mut Text, &mut Visibility), With<BuffStackNumber>>,
) {
    for stack_change in stack_events.iter() {
        if stack_change.target != spectating.0 {
            continue
        }
        for (buff_ui_entity, buff_id, mut despawn_timer) in buff_holders.iter_mut() {
            if buff_id.id != stack_change.id {
                continue
            }
            despawn_timer.0.reset();
            for descendant in children_query.iter_descendants(buff_ui_entity) {
                let Ok((mut text, mut vis)) = stacks.get_mut(descendant) else { continue };
                text.sections[0].value = stack_change.stacks.to_string();
                if stack_change.stacks != 1 {
                    *vis = Visibility::Visible;
                }
            }
            break // return cus we found the buff, dont return cus we want to go
                  // to next event
        }
    }
}

pub fn add_buffs(
    mut commands: Commands,
    mut buff_events: EventReader<BuffAddEvent>,
    spectating: Res<Spectating>,
    targets_query: Query<Entity, With<Player>>,
    buff_bar_ui: Query<Entity, With<BuffBar>>,
    debuff_bar_ui: Query<Entity, With<DebuffBar>>,
    fonts: Res<Fonts>,
    icons: Res<Icons>,
) {
    for event in buff_events.iter() {
        if event.target != spectating.0 {
            continue
        }
        let Ok(_) = targets_query.get(event.target) else { continue };
        let is_buff = event.bufftype == BuffType::Buff;
        let holder_ui = if is_buff {
            let Ok(buff_bar) = buff_bar_ui.get_single() else { continue };
            buff_bar
        } else {
            let Ok(debuff_bar) = debuff_bar_ui.get_single() else { continue };
            debuff_bar
        };
        commands.entity(holder_ui).with_children(|parent| {
            parent
                .spawn(buff_holder(event.duration, event.id.clone()))
                .with_children(|parent| {
                    parent.spawn(buff_timer(&fonts, is_buff));
                    parent.spawn(buff_border(is_buff)).with_children(|parent| {
                        parent.spawn(buff_image(Ability::Frostbolt, &icons));
                        parent.spawn(buff_stacks(&fonts));
                    });
                });
        });
    }
}

pub fn toggle_objective_health(
    focused_health_entity: Res<FocusedHealthEntity>,
    mut obj_health_holder: Query<&mut Visibility, With<ObjectiveHealth>>,
    mut obj_text: Query<&mut Text, With<ObjectiveName>>,
    objective_query: Query<&Name>,
) {
    if focused_health_entity.is_changed() {
        let Ok(mut vis) = obj_health_holder.get_single_mut() else { return };
        if let Some(focused_entity) = focused_health_entity.0 {
            let Ok(mut text) = obj_text.get_single_mut() else { return };
            let Ok(name) = objective_query.get(focused_entity) else { return };
            text.sections[0].value = name.as_str().to_string();
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

pub fn spawn_floating_damage(
    mut damage_events: EventReader<HealthMitigatedEvent>,
    spectating: Res<Spectating>,
    mut commands: Commands,
    damaged_query: Query<Entity>,
    fonts: Res<Fonts>,
) {
    for damage_instance in damage_events.iter() {
        if damage_instance.attacker != spectating.0 && damage_instance.defender != spectating.0 {
            continue
        }
        let Ok(damaged) = damaged_query.get(damage_instance.defender) else { continue };
        let mut color = Color::WHITE;
        if damage_instance.defender == spectating.0 {
            color = Color::RED;
        }
        commands
            .spawn(follow_wrapper(damaged))
            .with_children(|parent| {
                parent.spawn(follow_inner_text(
                    damage_instance.change.abs().to_string(),
                    &fonts,
                    color,
                ));
            });
    }
}

pub fn floating_damage_cleanup(
    mut commands: Commands,
    mut tween_events: EventReader<TweenCompleted>,
    parents: Query<&Parent>,
) {
    for ev in tween_events.iter() {
        use TweenEvents::*;
        match TweenEvents::try_from(ev.user_data) {
            Ok(FloatingDamageEnded) => {
                let Ok(parent) = parents.get(ev.entity) else { continue };
                commands.entity(parent.get()).despawn_recursive();
            }
            Err(_) | Ok(_) => (),
        }
    }
}

pub fn update_damage_log_ui(
    mut commands: Commands,
    mut damage_events: EventReader<LogHit>,
    spectating: Res<Spectating>,
    incoming_ui: Query<Entity, With<IncomingLogUi>>,
    outgoing_ui: Query<Entity, With<OutgoingLogUi>>,
    fonts: Res<Fonts>,
    images: Res<Images>,
    icons: Res<Icons>,
    entities: Query<(Option<&Tower>, Option<&Player>, &Name, &Team)>,
    mut log_holders: Query<(Entity, &DamageLogId, &mut DespawnTimer)>,
    children_query: Query<&Children>,
    mut entry_text: Query<(&mut Text, &mut StoredNumber, &EntryText, &DamageLogId)>,
) {
    for event in damage_events.iter() {
        let (log_ui, other_party, direction) = match event.log_direction {
            LogSide::Incoming => {
                if spectating.0 != event.defender {
                    continue
                }
                let Ok(incoming_ui) = incoming_ui.get_single() else { continue };
                (incoming_ui, event.attacker, "from".to_string())
            }
            LogSide::Outgoing => {
                if spectating.0 != event.attacker {
                    continue
                }
                let Ok(outgoing_ui) = outgoing_ui.get_single() else { continue };
                (outgoing_ui, event.defender, "to".to_string())
            }
        };

        if event.log_type == LogType::Add {
            let mut image: Handle<Image> = images.default.clone();
            let mut name = "".to_string();
            if let Ok((is_tower, is_player, attacker_name, team)) = entities.get(other_party) {
                if is_tower.is_some() {
                    image = images.enemy_tower.clone();
                } else if is_player.is_some() {
                    image = images.friendly_tower.clone();
                }
                name = attacker_name.as_str().to_string();
            }

            let sensor_image = event.ability.get_image(&icons);
            let mut mitigated_string = format!("(-{})", event.mitigated.clone() as i32);
            if event.mitigated == 0 {
                mitigated_string = "".to_string();
            }
            let change = event.change.abs();

            commands.entity(log_ui).with_children(|parent| {
                parent
                    .spawn(despawn_wrapper(30))
                    .insert(DamageLogId(event.sensor))
                    .with_children(|parent| {
                        //parent.spawn(damage_column()).insert().with_children(|parent| {
                        parent.spawn(damage_entry()).with_children(|parent| {
                            parent.spawn(custom_image(sensor_image, 24));
                            parent.spawn(plain_text(direction, 14, &fonts));
                            parent.spawn(thin_image(image));
                            parent.spawn(plain_text(name, 16, &fonts));
                            parent.spawn(plain_text("dealt".to_string(), 14, &fonts));
                            parent
                                .spawn(plain_text((change as u32).to_string(), 18, &fonts))
                                .insert((
                                    EntryText::Change,
                                    StoredNumber(change as i32),
                                    DamageLogId(other_party),
                                ));
                            parent
                                .spawn(color_text(
                                    mitigated_string,
                                    18,
                                    &fonts,
                                    event.damage_type.get_color(),
                                ))
                                .insert((
                                    EntryText::Mitigated,
                                    StoredNumber(event.mitigated as i32),
                                    DamageLogId(other_party),
                                ));
                            parent
                                .spawn(color_text("".to_string(), 16, &fonts, Color::YELLOW))
                                .insert((
                                    EntryText::Hits,
                                    StoredNumber(1 as i32),
                                    DamageLogId(other_party),
                                ));
                        });
                        // });
                    });
            });
        } else {
            for (log_ui_entity, log_id, mut despawn_timer) in log_holders.iter_mut() {
                if event.sensor != log_id.0 {
                    continue
                }
                despawn_timer.0.reset();

                for descendant in children_query.iter_descendants(log_ui_entity) {
                    let Ok((mut text, mut number, entry_text, log_id)) =
                        entry_text.get_mut(descendant)
                    else {
                        continue
                    };

                    if other_party != log_id.0 {
                        continue
                    }
                    let added;
                    match entry_text {
                        EntryText::Change => {
                            added = number.0 + event.change.abs() as i32;
                            text.sections[0].value = added.to_string();
                        }
                        EntryText::Mitigated => {
                            added = number.0 + event.mitigated as i32;
                            if event.mitigated > 0 {
                                text.sections[0].value = format!("(-{})", added);
                            }
                        }
                        EntryText::Hits => {
                            added = number.0 + 1;
                            text.sections[0].value = format!("x{}", added);
                        }
                    }
                    number.0 = added;
                }
            }
        }
    }
}

#[derive(Component, Debug)]
pub enum EntryText {
    Change,
    Mitigated,
    Hits,
}
#[derive(Component, Debug)]
pub struct StoredNumber(pub i32);

#[derive(Component, Debug)]
pub struct DamageLogId(pub Entity);

#[derive(Resource)]
pub struct FocusedHealthEntity(pub Option<Entity>);
