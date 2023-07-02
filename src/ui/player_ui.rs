use bevy::{prelude::*, ui::RelativeCursorPosition};
use bevy_tweening::TweenCompleted;

use crate::{
    ability::{Ability, AbilityInfo, HealthChangeEvent},
    assets::{Fonts, Icons},
    buff::{BuffAddEvent, BuffStackEvent, BuffType},
    input::SlotAbilityMap,
    player::{CooldownMap, Player, WindupTimer, CastEvent},
    stats::*,
    ui::ui_bundles::*,
    view::Spectating, crowd_control::{CCMap, CCType}, game_manager::AbilityFireEvent,
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
                    parent.spawn(bar_wrapper()).with_children(|parent| {
                        parent.spawn(hp_bar(20.0)).with_children(|parent| {
                            parent.spawn(hp_bar_color());
                            parent.spawn(hp_bar_inner()).with_children(|parent| {
                                parent.spawn(hp_bar_text(&fonts));
                            });
                        });
                        parent.spawn(hp_bar(14.0)).with_children(|parent| {
                            parent.spawn(resource_bar_color());
                            parent.spawn(hp_bar_inner()).with_children(|parent| {
                                parent.spawn(resource_bar_text(&fonts));
                            });
                        });
                    });
                    // CDs
                    parent.spawn((ability_holder(), ability_map.clone()));
                });
                // CC on self
                parent.spawn(cc_holder()).with_children(|parent| {
                    parent.spawn(cc_icon(CCType::Root, &icons,)).insert(CCIconSelf);
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
            });
        });
    }
}

pub fn add_ability_icons(
    mut commands: Commands,
    query: Query<(Entity, &SlotAbilityMap), Added<AbilityHolder>>, // Changed<AbilityHolder> for changing spells midgame
    icons: Res<Icons>,
    fonts: Res<Fonts>,
) {
    for (entity, ability_map) in query.iter() {
        for (_, ability) in &ability_map.map {
            //let image_path = format!("icons/{}.png", ability.to_string().to_lowercase());
            let ability_icon = commands
                .spawn((
                    ability_image(&icons, ability.clone()),
                    AbilityInfo::new(ability),
                    Hoverable,
                    RelativeCursorPosition::default(),
                ))
                .id();

            let cd_text = commands.spawn((cd_text(&fonts), ability.clone())).id();

            commands.entity(ability_icon).push_children(&[cd_text]);
            commands.entity(entity).push_children(&[ability_icon]);
        }
    }
}

// Change these to generics later, requires Bar<Health> and BarText<Health>
pub fn update_health(
    query: Query<&Attributes, (With<Player>, Changed<Attributes>)>,
    mut text_query: Query<&mut Text, With<HealthBarText>>,
    mut bar_query: Query<&mut Style, With<HealthBar>>,
    spectating: Res<Spectating>,
) {
    let Some(spectating) = spectating.0 else {return};
    match (text_query.get_single_mut(), bar_query.get_single_mut()) {
        (Ok(mut text), Ok(mut bar)) => {
            let Ok(attributes) = query.get(spectating) else { return };
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
            //let new_size = (current / max).to_num::<f32>();
            bar.size.width = Val::Percent(new_size * 100.0);
        }
        _ => {}
    }
}

pub fn update_character_resource(
    query: Query<&Attributes, (With<Player>, Changed<Attributes>)>,
    mut text_query: Query<&mut Text, With<ResourceBarText>>,
    mut bar_query: Query<&mut Style, With<ResourceBar>>,
    spectating: Res<Spectating>,
) {
    let Some(spectating) = spectating.0 else {return};
    match (text_query.get_single_mut(), bar_query.get_single_mut()) {
        (Ok(mut text), Ok(mut bar)) => {
            let Ok(attributes) = query.get(spectating) else { return };
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
            //let new_size = (current / max_amount).to_num::<f32>();
            bar.size.width = Val::Percent(new_size * 100.0);
        }
        _ => {}
    }
}

pub fn update_cc_bar(
    mut cc_bar_fill: Query<&mut Style, With<CCBarSelfFill>>,
    cc_maps: Query<&CCMap>,
    spectating: Res<Spectating>,
){
    let Some(spectating) = spectating.0 else {return};
    let Ok(cc_of_spectating) = cc_maps.get(spectating) else {return}; 
    let cc_vec = Vec::from_iter(cc_of_spectating.map.clone());
    let Some((_top_cc, cc_timer)) = cc_vec.get(0) else {return};
    let Ok(mut style) = cc_bar_fill.get_single_mut() else {return};
    style.size.width = Val::Percent(cc_timer.percent_left() * 100.0);    
}

pub fn toggle_cc_bar(
    mut cc_bar: Query<&mut Visibility, With<CCSelf>>,
    mut cc_icon: Query<&mut UiImage, With<CCIconSelf>>,
    icons: Res<Icons>,
    cc_maps: Query<&CCMap, Changed<CCMap>>,
    spectating: Res<Spectating>,
){
    let Some(spectating) = spectating.0 else {return};
    let Ok(cc_of_spectating) = cc_maps.get(spectating) else {return}; 
    let Ok(mut vis) = cc_bar.get_single_mut() else {return};
    let Ok(mut image) = cc_icon.get_single_mut() else {return};
    if cc_of_spectating.map.is_empty(){
        *vis = Visibility::Hidden;
    } else {
        *vis = Visibility::Visible;
        let cc_vec = Vec::from_iter(cc_of_spectating.map.clone());
        let Some((top_cc, _)) = cc_vec.get(0) else {return};
        image.texture = top_cc.clone().get_icon(&icons);
    }
}

pub fn update_cast_bar(
    mut cast_bar_fill: Query<&mut Style, With<CastBarFill>>,
    windup_query: Query<&WindupTimer>,
    spectating: Res<Spectating>,
){
    let Some(spectating) = spectating.0 else {return};
    let Ok(windup) = windup_query.get(spectating) else {return}; 
    let Ok(mut style) = cast_bar_fill.get_single_mut() else {return};
    style.size.width = Val::Percent(windup.0.percent() * 100.0);    
}

pub fn toggle_cast_bar(
    mut bar: Query<&mut Visibility, With<CastBar>>,
    mut cast_events: EventReader<CastEvent>,
    mut fire_events: EventReader<AbilityFireEvent>,
    spectating: Res<Spectating>,
){
    let Some(spectating) = spectating.0 else {return};
    let Ok(mut vis) = bar.get_single_mut() else {return};
    for event in cast_events.iter(){
        if event.caster != spectating {continue}
        *vis = Visibility::Visible;
    }
    for event in fire_events.iter(){
        if event.caster != spectating {continue}
        *vis = Visibility::Hidden;
    }
}


pub fn update_cooldowns(
    mut text_query: Query<(&mut Text, &Ability, &Parent), With<CooldownIconText>>,
    cooldown_query: Query<&CooldownMap>,
    mut image_query: Query<&mut BackgroundColor, With<UiImage>>,
    spectating: Res<Spectating>,
) {
    let Some(spectating) = spectating.0 else {return};
    let Ok(cooldowns) = cooldown_query.get(spectating) else {return};

    for (mut text, ability, parent) in text_query.iter_mut() {
        let Ok(mut background_color) = image_query.get_mut(parent.get()) else{
            continue
        };
        if !cooldowns.map.contains_key(ability) {
            text.sections[0].value = String::from("");
            *background_color = Color::WHITE.into();
        } else {
            let Some(timer) = cooldowns.map.get(ability) else {continue};
            let newcd = timer.remaining_secs() as u32;
            text.sections[0].value = newcd.to_string();
            *background_color = Color::rgb(0.2, 0.2, 0.2).into();
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
    let Some(spectating) = spectating.0 else {return};
    for stack_change in stack_events.iter() {
        if stack_change.target != spectating {
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
    targets_query: Query<Entity, (With<Player>)>,
    buff_bar_ui: Query<Entity, With<BuffBar>>,
    debuff_bar_ui: Query<Entity, With<DebuffBar>>,
    mut buff_events: EventReader<BuffAddEvent>,
    icons: Res<Icons>,
    fonts: Res<Fonts>,
    spectating: Res<Spectating>,
) {
    let Some(spectating) = spectating.0 else {return};
    for event in buff_events.iter() {
        if event.target != spectating {
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
    let Some(spectating) = spectating.0 else {return};
    for damage_instance in damage_events.iter(){
        let Some(attacker_entity) = damage_instance.attacker else {continue};
        if attacker_entity != spectating { continue };
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
