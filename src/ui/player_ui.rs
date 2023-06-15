use bevy::{prelude::*, ui::RelativeCursorPosition};


use crate::{
    ui::{ui_bundles::*, },
    player::{Player, CooldownMap}, 
    input::SlotAbilityMap, 
    ability::{AbilityInfo, Ability},
    stats::*, assets::{Icons, Fonts},    
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
                    parent.spawn(hp_bar(12.0)).with_children(|parent| {
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
) {
    match (text_query.get_single_mut(), bar_query.get_single_mut()) {
        (Ok(mut text), Ok(mut bar)) => {
            for (hp, regen, max) in query.iter() {
                let current_amount = *hp.amount();
                let regen_amount = *regen.amount();
                let max_amount = *max.amount();

                text.sections[0].value =
                    format!("{} / {} (+{})", current_amount.trunc(), max_amount.trunc(), regen_amount.trunc());

                let new_size = current_amount as f32 / max_amount as f32;
                //let new_size = (current_amount / max_amount).to_num::<f32>();
                bar.size.width = Val::Percent(new_size * 100.0);
            }
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
) {
    match (text_query.get_single_mut(), bar_query.get_single_mut()) {
        (Ok(mut text), Ok(mut bar)) => {
            for (resource, regen, max) in query.iter() {
                let current_amount = *resource.amount();
                let regen_amount = *regen.amount();
                let max_amount = *max.amount();

                text.sections[0].value =
                    format!("{} / {} (+{})", current_amount.trunc(), max_amount.trunc(), regen_amount.trunc());

                let new_size = current_amount as f32 / max_amount as f32;
                //let new_size = (current_amount / max_amount).to_num::<f32>();
                bar.size.width = Val::Percent(new_size * 100.0);
            }
        }
        _ => {}
    }
}

pub fn update_cooldowns(
    mut text_query: Query<(&mut Text, &Ability, &Parent), With<CooldownIconText>>,
    cooldown_query: Query<(&Player, &CooldownMap)>,
    mut image_query: Query<&mut BackgroundColor, With<UiImage>>,
){
    for (mut text, ability, parent) in text_query.iter_mut() {
        let Ok(mut background_color) = image_query.get_mut(parent.get()) else{
            continue
        };
        for (_, cooldowns) in cooldown_query.iter() {
            if !cooldowns.map.contains_key(ability) {
                text.sections[0].value = String::from("");
                *background_color = Color::WHITE.into();
            } else {
                let timer = cooldowns.map.get(ability).unwrap();
                let newcd = timer.remaining_secs() as u32;
                text.sections[0].value = newcd.to_string();
                *background_color = Color::rgb(0.2, 0.2, 0.2).into();
            }
        }
    }
}