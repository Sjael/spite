use bevy::{prelude::*, ui::RelativeCursorPosition};


use crate::{
    ui::{ui_bundles::*, },
    player::{Player, CooldownMap}, 
    input::SlotAbilityMap, 
    ability::{AbilityInfo, Ability},
    stats::*, assets::{Icons, Fonts}, buff::{BuffType, BuffStackEvent, BuffMap, BuffAddEvent}, view::Spectating,    
};

pub fn add_player_ui(
    mut commands: Commands,
    ui_query: Query<Entity, With<RootUI>>,
    player_query: Query<&SlotAbilityMap, Added<Player>>,
    fonts: Res<Fonts>,
) {
    let Ok(root_ui) = ui_query.get_single() else {return};
    for ability_map in player_query.iter() {  
        commands.entity(root_ui).with_children(|parent| {
            // Bottom Container
            parent.spawn(player_bottom_container())
            .with_children(|parent| {
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
                parent.spawn((
                    ability_holder(),
                    ability_map.clone(),
                ));
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
            let ability_icon = commands.spawn((
                ability_image(&icons, ability.clone()),
                AbilityInfo::new(ability),
                Hoverable,
                RelativeCursorPosition::default(),
            )).id();
        
            let cd_text = commands.spawn((
                cd_text(&fonts),
                ability.clone()
            )).id();
        
            commands.entity(ability_icon).push_children(&[cd_text]);
            commands.entity(entity).push_children(&[ability_icon]);
        }
    }
}

// Change these to generics later, requires Bar<Health> and BarText<Health>
pub fn update_health(
    query: Query<
        (
            &Attribute<Health>,
            &Attribute<Regen<Health>>,
            &Attribute<Max<Health>>,
        ),
        (
            Or<(
                Changed<Attribute<Health>>,
                Changed<Attribute<Regen<Health>>>,
                Changed<Attribute<Max<Health>>>,
            )>,
            With<Player> // TODO make player with Id for multiplayer
        ),
    >,
    mut text_query: Query<&mut Text, With<HealthBarText>>,
    mut bar_query: Query<&mut Style, With<HealthBar>>,
    spectating: Res<Spectating>,
) {
    let Some(spectating) = spectating.0 else {return};
    match (text_query.get_single_mut(), bar_query.get_single_mut()) {
        (Ok(mut text), Ok(mut bar)) => {
            let Ok((
                hp, 
                regen, 
                max
            )) = query.get(spectating) else { return };
            let current_amount = *hp.amount();
            let regen_amount = *regen.amount();
            let max_amount = *max.amount();

            text.sections[0].value =
                format!("{} / {} (+{})", current_amount.trunc(), max_amount.trunc(), regen_amount.trunc());

            let new_size = current_amount as f32 / max_amount as f32;
            //let new_size = (current_amount / max_amount).to_num::<f32>();
            bar.size.width = Val::Percent(new_size * 100.0);
        }
        _ => {}
    }
}

pub fn update_character_resource(
    query: Query<
        (
            &Attribute<CharacterResource>,
            &Attribute<Regen<CharacterResource>>,
            &Attribute<Max<CharacterResource>>,
        ),
        Or<(
            Changed<Attribute<CharacterResource>>,
            Changed<Attribute<Regen<CharacterResource>>>,
            Changed<Attribute<Max<CharacterResource>>>,
        )>,
    >,
    mut text_query: Query<&mut Text, With<ResourceBarText>>,
    mut bar_query: Query<&mut Style, With<ResourceBar>>,
    spectating: Res<Spectating>,
) {
    let Some(spectating) = spectating.0 else {return};
    match (text_query.get_single_mut(), bar_query.get_single_mut()) {
        (Ok(mut text), Ok(mut bar)) => {
            let Ok((
                resource, 
                regen, 
                max
            )) = query.get(spectating) else { return };
            let current_amount = *resource.amount();
            let regen_amount = *regen.amount();
            let max_amount = *max.amount();

            text.sections[0].value =
                format!("{} / {} (+{})", current_amount.trunc(), max_amount.trunc(), regen_amount.trunc());

            let new_size = current_amount as f32 / max_amount as f32;
            //let new_size = (current_amount / max_amount).to_num::<f32>();
            bar.size.width = Val::Percent(new_size * 100.0);
        }
        _ => {}
    }
}

pub fn update_cooldowns(
    mut text_query: Query<(&mut Text, &Ability, &Parent), With<CooldownIconText>>,
    cooldown_query: Query<&CooldownMap>,
    mut image_query: Query<&mut BackgroundColor, With<UiImage>>,
    spectating: Res<Spectating>,
){
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
){
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
){
    let Some(spectating) = spectating.0 else {return};
    for stack_change in stack_events.iter(){
        if stack_change.target != spectating { continue }
        for (buff_ui_entity, buff_id, mut despawn_timer) in buff_holders.iter_mut(){
            if buff_id.id != stack_change.id { continue }
            despawn_timer.0.reset();
            for descendant in children_query.iter_descendants(buff_ui_entity){
                let Ok((mut text, mut vis)) = stacks.get_mut(descendant) else {continue};
                text.sections[0].value = stack_change.stacks.to_string();
                if stack_change.stacks != 1{
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
){
    let Some(spectating) = spectating.0 else {return};
    for event in buff_events.iter(){
        if event.target != spectating { continue }
        let Ok(_) = targets_query.get(event.target) else {continue};
        let Ok(buff_bar) = buff_bar_ui.get_single() else {continue};
        let Ok(debuff_bar) = debuff_bar_ui.get_single() else {continue};
        let is_buff = event.bufftype == BuffType::Buff;
        let holder_ui: Entity;
        if is_buff{
            holder_ui = buff_bar;
        } else{
            holder_ui = debuff_bar;
        }
        commands.entity(holder_ui).with_children(|parent| {
            parent.spawn(buff_holder(event.duration, event.id.clone())).with_children(|parent| {
                parent.spawn(buff_timer(&fonts, is_buff));
                parent.spawn(buff_border(is_buff)).with_children(|parent| {
                    parent.spawn(buff_image(Ability::Frostbolt, &icons));
                    parent.spawn(buff_stacks(&fonts));
                });
            });
        });
    }
}


/* 
pub fn update_buffs(
    buff_query: Query<(&Player, &BuffMap)>,
    mut text_query: Query<(&mut Text, &Ability, &Parent), With<CooldownIconText>>,
    mut image_query: Query<&mut BackgroundColor, With<UiImage>>,
){
    for (_, buffs) in buff_query.iter() {
        if !buffs.map.contains_key(ability) {
            text.sections[0].value = String::from("");
            *background_color = Color::WHITE.into();
        } else {
            let timer = buffs.map.get(ability).unwrap();
            let newcd = timer.remaining_secs() as u32;
            text.sections[0].value = newcd.to_string();
            *background_color = Color::rgb(0.2, 0.2, 0.2).into();
        }
    }
    for (mut text, ability, parent) in text_query.iter_mut() {
        let Ok(mut background_color) = image_query.get_mut(parent.get()) else{
            continue
        };
    }
}
*/