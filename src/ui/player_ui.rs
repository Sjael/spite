use bevy::{prelude::*, ui::RelativeCursorPosition};
use bevy_tweening::TweenCompleted;

use crate::{
    ability::{Ability, AbilityTooltip},
    area::HealthChangeEvent,
    assets::{Fonts, Icons},
    buff::{BuffAddEvent, BuffStackEvent, BuffType},
    input::SlotAbilityMap,
    stats::*,
    ui::ui_bundles::*,
    actor::view::Spectating, crowd_control::{CCMap, CCType}, 
    game_manager::AbilityFireEvent, 
    actor::{WindupTimer, CastEvent, CooldownMap, player::Player},
};

pub fn add_player_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<RootUI>>,
    player_query: Query<&SlotAbilityMap, Added<Player>>,
    fonts: Res<Fonts>,
    icons: Res<Icons>,
) {
    let Ok(root_ui) = ui_query.get_single() else {return};
    for ability_map in player_query.iter() {
        commands.entity(root_ui).with_children(|parent| {
            parent.spawn(character_ui()).with_children(|parent| {
                // Bottom Container
                parent.spawn(player_bottom_container()).with_children(|parent| {
                    // Buffs / Debuffs
                    parent.spawn(effect_bar()).with_children(|parent| {
                        parent.spawn(buff_bar());
                        parent.spawn(debuff_bar());
                    });
                    // Resource Bars
                    parent.spawn(player_bars_wrapper()).with_children(|parent| {
                        parent.spawn(bar_wrapper(20.0)).with_children(|parent| {
                            parent.spawn(hp_bar_color());
                            parent.spawn(bar_text_wrapper()).with_children(|parent| {
                                parent.spawn(hp_bar_text(&fonts));
                            });
                        });
                        parent.spawn(bar_wrapper(14.0)).with_children(|parent| {
                            parent.spawn(resource_bar_color());
                            parent.spawn(bar_text_wrapper()).with_children(|parent| {
                                parent.spawn(resource_bar_text(&fonts));
                            });
                        });
                    });
                    // CDs
                    parent.spawn((ability_holder(), ability_map.clone()));
                });
                // CC on self
                parent.spawn(cc_holder()).with_children(|parent| {
                    parent.spawn(cc_holder_top()).with_children(|parent| {
                        parent.spawn(cc_icon(CCType::Root, &icons,)).insert(CCIconSelf);
                        parent.spawn(plain_text("", 24, &fonts)).insert(CCSelfLabel);
                    });
                    parent.spawn(cc_bar()).with_children(|parent| {
                        parent.spawn(cc_bar_fill());
                    });
                });
                // castbar
                parent.spawn(cast_bar_holder()).with_children(|parent| {
                    //parent.spawn(cc_icon(CCType::Root, &icons,)).insert(CCIconSelf);
                    parent.spawn(cast_bar()).with_children(|parent| {
                        parent.spawn(cast_bar_fill());
                    });
                });
                // objective health
                parent.spawn(objective_health_bar_holder()).with_children(|parent| {
                    parent.spawn(plain_text("", 18, &fonts)).insert(ObjectiveName);
                    parent.spawn(bar_wrapper(24.0)).with_children(|parent| {
                        parent.spawn(objective_health_fill());
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
            let ability_icon = commands.spawn((
                    ability_image(&icons, ability.clone()),
                    ability.get_tooltip(),
                    Hoverable,
                    RelativeCursorPosition::default(),
                )).id();

            let cd_text = commands.spawn((cd_text(&fonts), ability.clone())).id();

            commands.entity(cd_text).set_parent(ability_icon);
            commands.entity(ability_icon).set_parent(entity);
        }
    }
}


pub fn update_health(
    query: Query<&Attributes, (With<Player>, Changed<Attributes>)>,
    mut text_query: Query<&mut Text, With<HealthBarText>>,
    mut bar_query: Query<&mut Style, With<HealthBarUI>>,
    spectating: Res<Spectating>,
) {
    let Ok(mut text) = text_query.get_single_mut() else {return};
    let Ok(mut bar) = bar_query.get_single_mut() else {return};
    let Ok(attributes) = query.get(spectating.0) else { return };
    let current = *attributes.get(&Stat::Health.into()).unwrap_or(&0.0);
    let regen = *attributes.get(&Stat::HealthRegen.into()).unwrap_or(&0.0);
    let max = *attributes.get(&Stat::HealthMax.into()).unwrap_or(&100.0);

    text.sections[0].value = format!(
        "{} / {} (+{})",
        current.trunc(),
        max.trunc(),
        regen.trunc()
    );

    let new_size = current / max;
    bar.size.width = Val::Percent(new_size * 100.0);
}

pub fn update_character_resource(
    query: Query<&Attributes, (With<Player>, Changed<Attributes>)>,
    mut text_query: Query<&mut Text, With<ResourceBarText>>,
    mut bar_query: Query<&mut Style, With<ResourceBarUI>>,
    spectating: Res<Spectating>,
) {
    let Ok(mut text) = text_query.get_single_mut() else {return};
    let Ok(mut bar) = bar_query.get_single_mut() else {return};
    let Ok(attributes) = query.get(spectating.0) else { return };
    let current = *attributes.get(&Stat::CharacterResource.into()).unwrap_or(&0.0);
    let regen = *attributes.get(&Stat::CharacterResourceRegen.into()).unwrap_or(&0.0);
    let max = *attributes.get(&Stat::CharacterResourceMax.into()).unwrap_or(&100.0);

    text.sections[0].value = format!(
        "{} / {} (+{})",
        current.trunc(),
        max.trunc(),
        regen.trunc()
    );

    let new_size = current / max;
    bar.size.width = Val::Percent(new_size * 100.0);
}

pub fn update_objective_health(    
    query: Query<&Attributes, Changed<Attributes>>,
    focused_health_entity: Res<FocusedHealthEntity>,
    //mut text_query: Query<&mut Text, With<HealthBarText>>,
    mut bar_query: Query<&mut Style, With<ObjectiveHealthFill>>,
){
    let Ok(mut bar) = bar_query.get_single_mut() else {return};
    let Some(focused_entity) = focused_health_entity.0 else {return};
    let Ok(attributes) = query.get(focused_entity) else {return};
    let current = *attributes.get(&Stat::Health.into()).unwrap_or(&0.0);
    let max = *attributes.get(&Stat::HealthMax.into()).unwrap_or(&100.0);

    let new_size = current / max;
    bar.size.width = Val::Percent(new_size * 100.0);
}

pub fn toggle_objective_health(
    focused_health_entity: Res<FocusedHealthEntity>,
    objective_query: Query<(&Attributes, &Name)>,
    mut obj_health_holder: Query<&mut Visibility, With<ObjectiveHealth>>,
    mut obj_text: Query<&mut Text, With<ObjectiveName>>,
    mut bar_query: Query<&mut Style, With<ObjectiveHealthFill>>,
){
    if focused_health_entity.is_changed(){
        let Ok(mut vis) = obj_health_holder.get_single_mut() else {return};
        if let Some(focused_entity) = focused_health_entity.0{
            let Ok(mut bar) = bar_query.get_single_mut() else {return};
            let Ok(mut text) = obj_text.get_single_mut() else {return};
            let Ok((attributes, name)) = objective_query.get(focused_entity) else {return};
            let current = *attributes.get(&Stat::Health.into()).unwrap_or(&0.0);
            let max = *attributes.get(&Stat::HealthMax.into()).unwrap_or(&100.0);
            
            let new_size = current / max;
            //bar.size.width = Val::Percent(new_size * 100.0);
            text.sections[0].value = name.as_str().to_string();
            *vis = Visibility::Visible;
        } else{
            *vis = Visibility::Hidden;
        }
    }
}

pub fn update_cc_bar(
    mut cc_bar_fill: Query<&mut Style, With<CCBarSelfFill>>,
    cc_maps: Query<&CCMap>,
    spectating: Res<Spectating>,
){
    let Ok(cc_of_spectating) = cc_maps.get(spectating.0) else {return}; 
    let cc_vec = Vec::from_iter(cc_of_spectating.map.clone());
    let Some((_top_cc, cc_timer)) = cc_vec.get(0) else {return};
    let Ok(mut style) = cc_bar_fill.get_single_mut() else {return};
    style.size.width = Val::Percent(cc_timer.percent_left() * 100.0);    
}

pub fn toggle_cc_bar(
    mut cc_bar: Query<&mut Visibility, With<CCSelf>>,
    mut cc_icon: Query<&mut UiImage, With<CCIconSelf>>,
    mut cc_text: Query<&mut Text, With<CCSelfLabel>>,
    icons: Res<Icons>,
    cc_maps: Query<&CCMap, Changed<CCMap>>,
    spectating: Res<Spectating>,
){
    let Ok(cc_of_spectating) = cc_maps.get(spectating.0) else {return}; 
    let Ok(mut vis) = cc_bar.get_single_mut() else {return};
    let Ok(mut image) = cc_icon.get_single_mut() else {return};
    let Ok(mut text) = cc_text.get_single_mut() else {return};
    if cc_of_spectating.map.is_empty(){
        *vis = Visibility::Hidden;
    } else {
        *vis = Visibility::Visible;
        let cc_vec = Vec::from_iter(cc_of_spectating.map.clone());
        let Some((top_cc, _)) = cc_vec.get(0) else {return};
        image.texture = top_cc.clone().get_icon(&icons);
        text.sections[0].value = top_cc.to_text();
    }
}

pub fn update_cast_bar(
    mut cast_bar_fill: Query<&mut Style, With<CastBarFill>>,
    windup_query: Query<&WindupTimer>,
    spectating: Res<Spectating>,
){
    let Ok(windup) = windup_query.get(spectating.0) else {return}; 
    let Ok(mut style) = cast_bar_fill.get_single_mut() else {return};
    style.size.width = Val::Percent(windup.0.percent() * 100.0);    
}

pub fn toggle_cast_bar(
    mut bar: Query<&mut Visibility, With<CastBar>>,
    mut cast_events: EventReader<CastEvent>,
    mut fire_events: EventReader<AbilityFireEvent>,
    spectating: Res<Spectating>,
){
    let Ok(mut vis) = bar.get_single_mut() else {return};
    for event in cast_events.iter(){
        if event.caster != spectating.0 {continue}
        *vis = Visibility::Visible;
    }
    for event in fire_events.iter(){
        if event.caster != spectating.0 {continue}
        *vis = Visibility::Hidden;
    }
}


pub fn update_cooldowns(
    mut text_query: Query<(&mut Text, &Ability, &Parent), With<CooldownIconText>>,
    cooldown_query: Query<&CooldownMap>,
    cooldown_changed_query: Query<&CooldownMap, Changed<CooldownMap>>,
    mut image_query: Query<&mut BackgroundColor, With<UiImage>>,
    spectating: Res<Spectating>,
) {

    // tick existing cooldowns
    let Ok(cooldowns) = cooldown_query.get(spectating.0) else {return};
    for (mut text, ability, _) in text_query.iter_mut() {
        if cooldowns.map.contains_key(ability) {
            let Some(timer) = cooldowns.map.get(ability) else {continue};
            let newcd = timer.remaining_secs() as u32;
            text.sections[0].value = newcd.to_string();
        } 
    }
    // only when cooldowns change on spectating
    if let Ok(cooldowns_changed) = cooldown_changed_query.get(spectating.0){
        for (mut text, ability, parent) in text_query.iter_mut() {
            let Ok(mut background_color) = image_query.get_mut(parent.get()) else{ continue };
        
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
        let Ok(despawn_timer) = timer_query.get(parent.get()) else {continue};
        let remaining = despawn_timer.0.remaining_secs() as u32;
        text.sections[0].value = remaining.to_string();
    }
}

pub fn update_buff_stacks(
    mut stacks: Query<(&mut Text, &mut Visibility), With<BuffStackNumber>>,
    children_query: Query<&Children>,
    mut buff_holders: Query<(Entity, &BuffId, &mut DespawnTimer)>,
    mut stack_events: EventReader<BuffStackEvent>,
    spectating: Res<Spectating>,
) {
    for stack_change in stack_events.iter() {
        if stack_change.target != spectating.0 {
            continue;
        }
        for (buff_ui_entity, buff_id, mut despawn_timer) in buff_holders.iter_mut() {
            if buff_id.id != stack_change.id {
                continue;
            }
            despawn_timer.0.reset();
            for descendant in children_query.iter_descendants(buff_ui_entity) {
                let Ok((mut text, mut vis)) = stacks.get_mut(descendant) else {continue};
                text.sections[0].value = stack_change.stacks.to_string();
                if stack_change.stacks != 1 {
                    *vis = Visibility::Visible;
                }
            }
        }
    }
}

pub fn add_buffs(
    mut commands: Commands,
    targets_query: Query<Entity, With<Player>>,
    buff_bar_ui: Query<Entity, With<BuffBar>>,
    debuff_bar_ui: Query<Entity, With<DebuffBar>>,
    mut buff_events: EventReader<BuffAddEvent>,
    icons: Res<Icons>,
    fonts: Res<Fonts>,
    spectating: Res<Spectating>,
) {
    for event in buff_events.iter() {
        if event.target != spectating.0 {
            continue;
        }
        let Ok(_) = targets_query.get(event.target) else {continue};
        let Ok(buff_bar) = buff_bar_ui.get_single() else {continue};
        let Ok(debuff_bar) = debuff_bar_ui.get_single() else {continue};
        let is_buff = event.bufftype == BuffType::Buff;
        let holder_ui: Entity;
        if is_buff {
            holder_ui = buff_bar;
        } else {
            holder_ui = debuff_bar;
        }
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

pub fn spawn_floating_damage(
    mut damage_events: EventReader<HealthChangeEvent>,
    spectating: Res<Spectating>,
    mut commands: Commands,
    damaged_query: Query<Entity>,
    fonts: Res<Fonts>,
){
    for damage_instance in damage_events.iter(){
        let Some(attacker_entity) = damage_instance.attacker else {continue};
        if attacker_entity != spectating.0 { continue };
        let Ok(damaged) = damaged_query.get(damage_instance.defender) else {continue};
        commands.spawn(follow_wrapper(damaged)).with_children(|parent| {            
            parent.spawn(follow_inner_text(damage_instance.amount.abs().trunc().to_string(), &fonts));
        });
    }
}


pub fn floating_damage_cleanup(
    mut commands: Commands,
    mut tween_events: EventReader<TweenCompleted>,
    parents: Query<&Parent>,
){
    for ev in tween_events.iter(){
        use TweenEvents::*;
        match TweenEvents::try_from(ev.user_data) {
            Ok(FloatingDamageEnded) => {
                let Ok(parent) = parents.get(ev.entity) else {continue};
                commands.entity(parent.get()).despawn_recursive();  
            }
            Err(_) | Ok(_) => (),
        }
    }
}

pub fn update_damage_log_ui(
    incoming_ui: Query<Entity, With<IncomingLogUi>>,
    outgoing_ui: Query<Entity, With<OutgoingLogUi>>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    mut damage_events: EventReader<HealthChangeEvent>,
    spectating: Res<Spectating>,
){
    let Ok(incoming_log_entity) = incoming_ui.get_single() else {return};
    let Ok(outgoing_log_entity) = outgoing_ui.get_single() else {return};
    
    for damage_instance in damage_events.iter(){
        let mitigated = 9.to_string(); // TODO mitigated damage
        if let Some(attacker) = damage_instance.attacker{
            if spectating.0 == attacker{
                commands.entity(outgoing_log_entity).with_children(|parent| {
                    let defender_string = format!("{}v{}", damage_instance.defender.index(), damage_instance.defender.generation());
                    let entry_text = format!("{} - {} ({})", defender_string, damage_instance.amount.abs() as u32, mitigated);
                    parent.spawn(damage_entry(entry_text, &fonts));
                });
            }
        }
        if spectating.0 == damage_instance.defender{
            commands.entity(incoming_log_entity).with_children(|parent| {
                let mut attacker_string = "0v0".to_string();
                if let Some(attacker) = damage_instance.attacker{
                    attacker_string = format!("{}v{}", attacker.index(), attacker.generation());
                }
                let entry_text = format!("{} - {} ({})", attacker_string, damage_instance.amount.abs() as u32, mitigated);
                parent.spawn(damage_entry(entry_text, &fonts));
            });
        }
    } 
}


#[derive(Resource)]
pub struct FocusedHealthEntity(pub Option<Entity>);