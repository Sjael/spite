use bevy::{audio::Volume, prelude::*, ui::RelativeCursorPosition};
use bevy_tweening::TweenCompleted;

use crate::{
    ability::Ability,
    actor::{
        cast::{AbilitySlots, CooldownMap, Tower},
        log::{LogHit, LogSide, LogType},
        player::{LocalPlayer, LocalPlayerId, Player},
    },
    area::queue::area_apply_tags,
    assets::{Audio, Fonts, Icons, Images, Items},
    buff::{BuffAddEvent, BuffStackEvent, BuffType},
    camera::{PlayerBoom, Spectating},
    classes::warrior::gen_fury,
    crowd_control::{CCKind, CCMap},
    item::{Item, ITEM_DB},
    prelude::{ActorState, ActorType, InGameSet, Previous},
    session::{director::Respawns, team::*},
    stats::*,
    ui::{store::CATEGORIES, tooltip::Hoverable, ui_bundles::*, BarTrack, ButtonAction, TextTrack, FURY, GRAY},
};

pub fn build_spectating(app: &mut App) {
    app.insert_resource(FocusedHealthEntity(None));
    app.add_systems(
        Update,
        (
            add_player_ui,
            show_respawn_ui,
            tick_respawn_ui,
            toggle_cc_bar,
            update_cc_bar,
            // toggle_cast_bar,
            // update_cast_bar,
            add_ability_icons,
            update_cooldowns,
            add_buffs,
            update_buff_timers,
            update_buff_stacks,
            spawn_floating_damage,
            update_damage_log_ui,
            floating_damage_cleanup,
            //update_objective_health,
            toggle_objective_health,
            init_resource_pips_max.run_if(resource_exists::<LocalPlayer>),
            change_resource_pips_max.run_if(resource_exists::<LocalPlayer>),
            update_pips
                .run_if(resource_exists::<LocalPlayer>)
                .after(area_apply_tags)
                .after(gen_fury),
        )
            .in_set(InGameSet::Update),
    );
}

fn add_player_ui(
    mut commands: Commands,
    local_player_id: Option<Res<LocalPlayerId>>,
    ui_query: Query<Entity, With<RootUI>>,
    player_query: Query<(Entity, &AbilitySlots)>,
    cam_query: Query<(Entity, &PlayerBoom, &Spectating), Changed<Spectating>>,
    // cam_added: Query<(), Added<Spectating>>,
    fonts: Res<Fonts>,
    icons: Res<Icons>,
    items: Res<Items>,
) {
    let Ok(root_ui) = ui_query.get_single() else { return };
    let Some(local_player_id) = local_player_id else { return };
    for (_cam_entity, player_boom, spectating) in cam_query.iter() {
        if *local_player_id != **player_boom {
            // only when player is the camera's focus
            continue
        }
        let Some(spec) = spectating.get() else { continue };
        let Ok((entity, ability_slots)) = player_query.get(spec) else { continue };
        commands
            .spawn(character_ui())
            .with_children(|parent| {
                // Bottom Container
                parent.spawn(player_bottom_container()).with_children(|parent| {
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
                                .insert(BarTrack::hp(entity));
                            parent.spawn(bar_text_wrapper()).with_children(|parent| {
                                parent
                                    .spawn(custom_text(&fonts, 18.0, -1.0))
                                    .insert(TextTrack::new(entity, Stat::Health));
                            });
                        });
                        parent.spawn(resource_holder());
                        // parent.spawn(bar_background(14.0)).with_children(|parent| {
                        //     parent
                        //         .spawn(bar_fill(Color::rgb(0.92, 0.24, 0.01)))
                        //         .insert(BarTrack::res(entity));

                        //     parent.spawn(bar_text_wrapper()).with_children(|parent| {
                        //         parent
                        //             .spawn(custom_text(&fonts, 14.0, -2.0))
                        //             .insert(TextTrack::new(entity, Stat::CharacterResource));
                        //     });
                        // });
                    });
                    // CDs
                    parent.spawn((ability_holder(), ability_slots.clone()));
                });
                // CC on self
                parent.spawn(cc_holder()).with_children(|parent| {
                    parent.spawn(cc_holder_top()).with_children(|parent| {
                        parent.spawn(cc_icon(CCKind::Root, &icons)).insert(CCIconSelf);
                        parent.spawn(plain_text("", 24, &fonts)).insert(CCSelfLabel);
                    });
                    parent.spawn(bar_background(6.0)).with_children(|parent| {
                        parent.spawn(bar_fill(Color::WHITE.with_a(0.9))).insert(CCBarSelfFill);
                    });
                });
                // castbar
                parent.spawn(cast_bar_holder()).with_children(|parent| {
                    //parent.spawn(cc_icon(CCKind::Root, &icons,)).insert(CCIconSelf);
                    parent.spawn(bar_background(2.0)).with_children(|parent| {
                        parent.spawn(bar_fill(Color::YELLOW.with_a(0.9))).insert(CastBarFill);
                    });
                });
                // objective health
                parent.spawn(objective_health_bar_holder()).with_children(|parent| {
                    parent.spawn(plain_text("", 18, &fonts)).insert(ObjectiveName);
                    parent.spawn(bar_background(24.0)).with_children(|parent| {
                        parent
                            .spawn(bar_fill(Color::rgba(1.0, 0.2, 0.2, 0.9)))
                            .insert(ObjectiveHealthFill);
                    });
                });
                // Stats and build
                parent.spawn(bottom_left_ui_holder()).with_children(|parent| {
                    parent
                        .spawn((editable_ui_wrapper(), EditableUI::BottomLeft))
                        .with_children(|parent| {
                            parent.spawn(bottom_left_ui()).with_children(|parent| {
                                parent.spawn(stats_ui()).with_children(|parent| {
                                    for stat in LISTED_STATS.iter() {
                                        // add stat name or icon?
                                        //parent.spawn(plain_text(format!("stat {}", x), 16, &fonts));
                                        parent
                                            .spawn(plain_text("0", 16, &fonts))
                                            .insert(TextTrack::new(entity, stat.clone()));
                                    }
                                });
                                parent.spawn(inventory_and_kda()).with_children(|parent| {
                                    parent.spawn(kda_ui()).with_children(|parent| {
                                        parent.spawn(plain_text("0 / 0 / 0", 18, &fonts)).insert(PersonalKDA);
                                    });
                                    parent.spawn(inventory_ui()).with_children(|parent| {
                                        for i in 1..=6 {
                                            parent.spawn(build_slot(i));
                                        }
                                    });
                                });
                            });
                        });
                });
                parent.spawn(respawn_holder()).with_children(|parent| {
                    parent
                        .spawn((editable_ui_wrapper(), EditableUI::RespawnTimer))
                        .with_children(|parent| {
                            parent.spawn(respawn_text(&fonts));
                        });
                });
                parent.spawn(tab_panel()).with_children(|parent| {
                    parent.spawn(damage_log()).with_children(|parent| {
                        parent.spawn(log_outgoing());
                        parent.spawn(log_incoming());
                    });
                    parent.spawn(scoreboard());
                    parent.spawn(death_recap());
                    parent.spawn(abilities_panel());
                });
                parent.spawn(store()).with_children(|parent| {
                    parent.spawn(drag_bar());
                    parent.spawn(gold_holder()).with_children(|parent| {
                        parent
                            .spawn(color_text("0", 20, &fonts, Color::YELLOW))
                            .insert((TextTrack::new(entity, Stat::Gold), ZIndex::Global(4)));
                    });
                    parent.spawn(list_categories()).with_children(|parent| {
                        for stat in CATEGORIES.iter() {
                            parent.spawn(category(stat.clone())).with_children(|parent| {
                                parent.spawn(category_text(stat.to_string(), &fonts));
                            });
                        }
                    });
                    parent.spawn(list_items()).with_children(|parent| {
                        for item in ITEM_DB.keys() {
                            parent.spawn(store_item_wrap(item.clone())).with_children(|parent| {
                                parent.spawn(store_item(&items, item.clone()));
                                parent
                                    .spawn(color_text("", 16, &fonts, Color::WHITE))
                                    .insert(ItemDiscount(item.clone()));
                            });
                        }
                    });
                    parent.spawn(inspector()).with_children(|parent| {
                        parent.spawn(item_parents());
                        parent.spawn(grow_wrap()).with_children(|parent| {
                            parent.spawn(item_tree());
                        });
                        parent.spawn(item_details()).with_children(|parent| {
                            parent
                                .spawn(color_text("", 14, &fonts, Color::YELLOW))
                                .insert(ItemPriceText);
                            parent
                                .spawn(color_text("", 16, &fonts, Color::GREEN))
                                .insert((ItemDiscount(Item::Arondight), ItemDiscountText));
                            parent
                                .spawn(color_text("", 18, &fonts, Color::WHITE))
                                .insert(ItemNameText);
                            parent.spawn(hori()).with_children(|parent| {
                                parent
                                    .spawn(button())
                                    .insert(ButtonAction::BuyItem)
                                    .with_children(|parent| {
                                        parent.spawn(plain_text("BUY", 20, &fonts));
                                    });
                                parent
                                    .spawn(button())
                                    .insert(ButtonAction::SellItem)
                                    .with_children(|parent| {
                                        parent.spawn(plain_text("SELL", 16, &fonts));
                                    });
                            });
                            parent
                                .spawn(button())
                                .insert(ButtonAction::UndoStore)
                                .with_children(|parent| {
                                    parent.spawn(plain_text("UNDO", 16, &fonts));
                                });
                        });
                    });
                });
            })
            .set_parent(root_ui);
    }
}

fn show_respawn_ui(
    mut death_timer: Query<&mut Visibility, With<RespawnHolder>>,
    changed_states: Query<&ActorState, Changed<ActorState>>,
    local_entity: Option<Res<LocalPlayer>>,
) {
    let Ok(mut vis) = death_timer.get_single_mut() else { return };
    let Some(player) = local_entity else { return };
    let Ok(actor_state) = changed_states.get(**player) else { return };
    if actor_state.is_dead() {
        *vis = Visibility::Visible;
    } else {
        *vis = Visibility::Hidden;
    }
}

fn tick_respawn_ui(
    mut death_timer: Query<&mut Text, With<RespawnText>>,
    respawning: Res<Respawns>,
    local_id: Option<Res<LocalPlayerId>>,
) {
    let Ok(mut respawn_text) = death_timer.get_single_mut() else { return };
    let Some(local) = local_id else { return };
    let Some(respawn) = respawning.map.get(&ActorType::Player(**local)) else { return };
    let new_text = (respawn.duration().as_secs() as f32 - respawn.elapsed_secs()).floor() as u64;
    respawn_text.sections[1].value = new_text.to_string();
}

fn toggle_cc_bar(
    player: Option<Res<LocalPlayer>>,
    cc_maps: Query<&CCMap, Changed<CCMap>>,
    mut cc_bar: Query<&mut Visibility, With<CCSelf>>,
    mut cc_icon: Query<&mut UiImage, With<CCIconSelf>>,
    mut cc_text: Query<&mut Text, With<CCSelfLabel>>,
    icons: Res<Icons>,
) {
    let Some(player) = player else { return };

    let Ok(cc_of_spectating) = cc_maps.get(**player) else { return };
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

// TODO: Fix all of these to update based on the focused entity.
fn update_cc_bar(
    player: Option<Res<LocalPlayer>>,
    cc_maps: Query<&CCMap>,
    mut cc_bar_fill: Query<&mut Style, With<CCBarSelfFill>>,
) {
    let Some(player) = player else { return };
    let Ok(cc_of_spectating) = cc_maps.get(**player) else { return };
    let cc_vec = Vec::from_iter(cc_of_spectating.map.clone());
    let Some((_, cc_timer)) = cc_vec.get(0) else { return };
    let Ok(mut bar) = cc_bar_fill.get_single_mut() else { return };
    bar.width = Val::Percent(cc_timer.fraction_remaining() * 100.0);
}

// fn toggle_cast_bar(
//     player: Option<Res<LocalPlayer>>,
//     mut bar: Query<&mut Visibility, With<CastBar>>,
//     mut fire_events: EventReader<AbilityFireEvent>,
//     casters: Query<&Casting, Changed<Casting>>,
// ) {
//     let Some(player) = player else { return };
//     let Ok(mut vis) = bar.get_single_mut() else { return };
//     let Ok(casting) = casters.get(*player) else {return};
// }

// fn update_cast_bar(player: Option<Res<LocalPlayer>>, mut cast_bar_fill: Query<&mut Style, With<CastBarFill>>) {
//     let Some(player) = player else { return };
//     let Ok(mut style) = cast_bar_fill.get_single_mut() else { return };
//     style.width = Val::Percent(windup.0.fraction() * 100.0);
// }

fn init_resource_pips_max(
    mut commands: Commands,
    ui_query: Query<Entity, Added<ResourceHolder>>,
    actors: Query<&Attributes>,
    player: Res<LocalPlayer>,
) {
    let Ok(holder) = ui_query.get_single() else { return };
    let Ok(attrs) = actors.get(**player) else { return };
    let new = attrs.get(Stat::CharacterResourceMax).trunc() as i32;
    for i in 0..new {
        let pip = commands.spawn(resource_pip(i as u32)).id();
        commands.entity(pip).set_parent(holder);
    }
}

fn change_resource_pips_max(
    mut commands: Commands,
    ui_query: Query<(Entity, Option<&Children>), With<ResourceHolder>>,
    added: Query<(), Added<ResourceHolder>>,
    actors: Query<(&Attributes, &Previous<Attributes>), Changed<Attributes>>,
    player: Res<LocalPlayer>,
) {
    let Ok((holder, children)) = ui_query.get_single() else { return };
    let Ok((attrs, previous)) = actors.get(**player) else { return };
    let new = attrs.get(Stat::CharacterResourceMax).trunc() as i32;
    let old = previous.get(Stat::CharacterResourceMax).trunc() as i32;
    if old == new || added.get(holder).is_ok() {
        return
    }
    let diff = new - old;
    if diff > 0 {
        for i in old..new {
            let pip = commands.spawn(resource_pip(i as u32)).id();
            commands.entity(pip).set_parent(holder);
        }
    } else {
        let Some(children) = children else { return };
        for i in 0..diff.abs() as usize {
            let Some(child) = children.iter().rev().nth(i) else { continue };
            commands.entity(*child).despawn_recursive();
        }
    }
}

fn update_pips(
    mut commands: Commands,
    mut pips: Query<(&Pip, &mut BackgroundColor)>,
    actors: Query<(&Attributes, &Previous<Attributes>), Changed<Attributes>>,
    player: Res<LocalPlayer>,
    audio: Res<Audio>,
) {
    let Ok((attrs, previous)) = actors.get(**player) else { return };
    let new = attrs.get(Stat::CharacterResource) as u32;
    let old = previous.get(Stat::CharacterResource) as u32;
    let max = attrs.get(Stat::CharacterResourceMax) as u32;
    if old == new {
        return
    }
    if new == max && old <= new {
        // second cond is so if you lose resource max stat while maxxed it doesnt make sound
        commands.spawn(AudioBundle {
            source: audio.blip.clone(),
            settings: PlaybackSettings {
                volume: Volume::new(0.1),
                ..default()
            },
        });
    }
    for (pip, mut bg) in pips.iter_mut() {
        if pip.0 >= new {
            *bg = GRAY.into();
            continue
        }
        *bg = FURY.into();
    }
}

fn add_ability_icons(
    mut commands: Commands,
    query: Query<(Entity, &AbilitySlots), (With<AbilityHolder>, Changed<AbilitySlots>)>, // Changed<AbilityHolder> for changing spells midgame
    icons: Res<Icons>,
    fonts: Res<Fonts>,
) {
    for (entity, ability_slots) in query.iter() {
        for ability in ability_slots.abilities() {
            let ability_icon = commands
                .spawn((
                    ability_image(ability.get_image(&icons)),
                    Hoverable::Ability(ability),
                    RelativeCursorPosition::default(),
                ))
                .id();

            let cd_text = commands.spawn((cd_text(&fonts), ability.clone())).id();

            commands.entity(cd_text).set_parent(ability_icon);
            commands.entity(ability_icon).set_parent(entity);
        }
    }
}

fn update_cooldowns(
    player: Option<Res<LocalPlayer>>,
    cooldown_query: Query<&CooldownMap>,
    cooldown_changed_query: Query<&CooldownMap, Changed<CooldownMap>>,
    mut text_query: Query<(&mut Text, &Ability, &Parent), With<CooldownIconText>>,
    mut image_query: Query<&mut BackgroundColor, With<UiImage>>,
) {
    let Some(player) = player else { return };
    // tick existing cooldowns
    let Ok(cooldowns) = cooldown_query.get(**player) else { return };
    for (mut text, ability, _) in text_query.iter_mut() {
        if cooldowns.map.contains_key(ability) {
            let Some(timer) = cooldowns.map.get(ability) else { continue };
            let newcd = timer.remaining_secs() as u32;
            text.sections[0].value = newcd.to_string();
        }
    }
    // set bg color only when cooldowns change
    if let Ok(cooldowns_changed) = cooldown_changed_query.get(**player) {
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

fn add_buffs(
    mut commands: Commands,
    mut buff_events: EventReader<BuffAddEvent>,
    player: Option<Res<LocalPlayer>>,
    targets_query: Query<Entity, With<Player>>,
    buff_bar_ui: Query<Entity, With<BuffBar>>,
    debuff_bar_ui: Query<Entity, With<DebuffBar>>,
    fonts: Res<Fonts>,
) {
    let Some(player) = player else { return };
    for event in buff_events.read() {
        if event.target != *player {
            continue
        }
        let Ok(_) = targets_query.get(event.target) else { continue };
        let is_buff = event.info.bufftype == BuffType::Buff;
        let holder_ui = if is_buff {
            let Ok(buff_bar) = buff_bar_ui.get_single() else { continue };
            buff_bar
        } else {
            let Ok(debuff_bar) = debuff_bar_ui.get_single() else { continue };
            debuff_bar
        };
        commands.entity(holder_ui).with_children(|parent| {
            parent
                .spawn((
                    buff_holder(event.info.duration, event.id.clone()),
                    Hoverable::Buff(event.info.clone()),
                ))
                .with_children(|parent| {
                    parent.spawn(buff_timer(&fonts));
                    parent.spawn(buff_wrap(32, is_buff)).with_children(|parent| {
                        parent.spawn(buff_image(event.info.image.clone(), is_buff));
                        parent.spawn(buff_stacks(&fonts));
                    });
                });
        });
    }
}

fn update_buff_timers(
    mut text_query: Query<(&mut Text, &Parent), With<BuffDurationText>>,
    timer_query: Query<&DespawnTimer>,
) {
    for (mut text, parent) in text_query.iter_mut() {
        let Ok(despawn_timer) = timer_query.get(parent.get()) else { continue };
        let remaining = despawn_timer.0.remaining_secs() as u32;
        text.sections[0].value = remaining.to_string();
    }
}

fn update_buff_stacks(
    player: Option<Res<LocalPlayer>>,
    mut stack_events: EventReader<BuffStackEvent>,
    mut buff_holders: Query<(Entity, &BuffId, &mut DespawnTimer)>,
    children_query: Query<&Children>,
    mut stacks: Query<(&mut Text, &mut Visibility), With<BuffStackNumber>>,
) {
    let Some(player) = player else { return };
    for stack_change in stack_events.read() {
        if stack_change.target != *player {
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

fn toggle_objective_health(
    mut commands: Commands,
    focused_health_entity: Res<FocusedHealthEntity>,
    mut obj_health_holder: Query<&mut Visibility, With<ObjectiveHealth>>,
    mut obj_text: Query<&mut Text, With<ObjectiveName>>,
    mut obj_bar: Query<Entity, With<ObjectiveHealthFill>>,
    objective_query: Query<&Name>,
) {
    if focused_health_entity.is_changed() {
        let Ok(mut vis) = obj_health_holder.get_single_mut() else { return };
        let Ok(entity) = obj_bar.get_single_mut() else { return };
        if let Some(focused_entity) = focused_health_entity.0 {
            commands.entity(entity).insert(BarTrack::hp(focused_entity));

            let Ok(mut text) = obj_text.get_single_mut() else { return };
            let Ok(name) = objective_query.get(focused_entity) else { return };
            text.sections[0].value = name.as_str().to_string();
            *vis = Visibility::Visible;
        } else {
            *vis = Visibility::Hidden;
        }
    }
}

fn spawn_floating_damage(
    mut damage_events: EventReader<HealthMitigatedEvent>,
    local_player: Option<Res<LocalPlayer>>,
    mut commands: Commands,
    damaged_query: Query<Entity>,
    fonts: Res<Fonts>,
) {
    let Some(player) = local_player else { return };
    for damage_instance in damage_events.read() {
        let mut text = damage_instance.change.abs().to_string();
        if damage_instance.attacker != *player && damage_instance.defender != *player {
            continue
        }
        let Ok(damaged) = damaged_query.get(damage_instance.defender) else { continue };
        let mut color = Color::WHITE;
        if damage_instance.change == 0 {
            text = "immune".to_owned();
        } else if damage_instance.change > 0 {
            color = Color::GREEN;
        } else if damage_instance.defender == *player {
            color = Color::RED;
        }
        commands.spawn(follow_wrapper(damaged)).with_children(|parent| {
            parent.spawn(follow_inner_text(text, &fonts, color));
        });
    }
}

fn floating_damage_cleanup(
    mut commands: Commands,
    mut tween_events: EventReader<TweenCompleted>,
    parents: Query<&Parent>,
) {
    for ev in tween_events.read() {
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

fn update_damage_log_ui(
    mut commands: Commands,
    mut damage_events: EventReader<LogHit>,
    player: Option<Res<LocalPlayer>>,
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
    let Some(player) = player else { return };
    for event in damage_events.read() {
        let (log_ui, other_party, direction) = match event.log_direction {
            LogSide::Incoming => {
                if *player != event.defender {
                    continue
                }
                let Ok(incoming_ui) = incoming_ui.get_single() else { continue };
                (incoming_ui, event.attacker, "from".to_string())
            }
            LogSide::Outgoing => {
                if *player != event.attacker {
                    continue
                }
                let Ok(outgoing_ui) = outgoing_ui.get_single() else { continue };
                (outgoing_ui, event.defender, "to".to_string())
            }
        };

        if event.log_type == LogType::Add {
            let mut image: Handle<Image> = images.default.clone();
            let mut name = "".to_string();
            if let Ok((is_tower, is_player, attacker_name, _team)) = entities.get(other_party) {
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
                    let Ok((mut text, mut number, entry_text, log_id)) = entry_text.get_mut(descendant) else {
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
