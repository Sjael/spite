use bevy::{prelude::*, window::PrimaryWindow, app::AppExit};
use bevy_tweening::TweenCompleted;
use ui_bundles::team_thumbs_holder;

use crate::{
    ui::{ui_bundles::*,styles::*, spectating::*, mouse::*, ingame_menu::*, main_menu::*, hud_editor::*},  
    ability::AbilityTooltip,
    game_manager::{GameModeDetails, DeathEvent, Team, InGameSet, Scoreboard, ActorType, TeamRoster, TEAM_1}, 
    assets::{Icons, Items, Fonts, Images}, GameState, item::Item, 
    actor::{view::{PlayerCam, Spectating}, HasHealthBar, stats::{Attributes, Stat, AttributeTag}, player::Player},     
};

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SpectatingSet;

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FreeMouseSet;

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MouseState>();
        app.add_state::<TabMenu>();
        app.add_state::<StoreMenu>();
        app.add_state::<InGameMenu>();
        app.add_state::<EditingHUD>();

        app.add_event::<ResetUiEvent>();
        app.add_event::<MenuEvent>();

        app.insert_resource(FocusedHealthEntity(None));


        app.add_systems(OnEnter(GameState::MainMenu), (
            spawn_main_menu, 
        ));
        app.add_systems(Update, (
            exit_game_main_menu.run_if(in_state(GameState::MainMenu)),
        ));
        app.add_systems(OnExit(GameState::MainMenu), (
            cleanup, 
        ));

        app.add_systems(Update, (
            button_hovers,
            button_actions,
        ));

        app.add_systems(OnEnter(GameState::InGame), (
            add_base_ui,
            add_ingame_menu,
        ));
        
        app.configure_set(Update, SpectatingSet.run_if(resource_exists::<Spectating>()).run_if(in_state(GameState::InGame)));
        app.configure_set(Update, FreeMouseSet.run_if(in_state(MouseState::Free)).run_if(in_state(GameState::InGame)));

        app.add_systems(Update, (
            update_health,
            update_character_resource,
            update_cc_bar,
            toggle_cc_bar,
            update_cast_bar,
            toggle_cast_bar,
            update_cooldowns,
            add_buffs,
            update_buff_stacks,
            spawn_floating_damage,   
            update_damage_log_ui, 
        ).in_set(SpectatingSet));
        app.add_systems(Update, (
            add_player_ui,
            add_ability_icons,
            follow_in_3d,
            floating_damage_cleanup,
            update_buff_timers,
            update_objective_health,
            toggle_objective_health,
            populate_scoreboard,
        ).in_set(InGameSet::Update));
        app.add_systems(Update, (
            draggables.run_if(in_state(MouseState::Free)),
            menu_toggle,
            mouse_with_free_key,
            free_mouse,
            tick_despawn_timers,
            minimap_track,
            tick_clock_ui,       
            killfeed_update,
            kill_notif_cleanup,
            //show_floating_health_bars.run_if(resource_exists::<Spectating>()),
            spawn_floating_health_bars,
            bar_track,
            state_ingame_menu,
            update_kda,
        ).in_set(InGameSet::Update));
        
        app.add_systems(Update, (
            load_tooltip,
            move_tooltip,
        ).in_set(FreeMouseSet));
        app.add_systems(OnExit(MouseState::Free), hide_tooltip);

        app.add_systems(OnEnter(InGameMenu::Open), toggle_ingame_menu);
        app.add_systems(OnEnter(InGameMenu::Closed), toggle_ingame_menu);
        

        app.add_systems(OnEnter(EditingHUD::Yes), (
            give_editable_ui,
            scale_ui,
            reset_editable_ui,
        ));

        app.add_systems(OnEnter(EditingHUD::No), remove_editable_ui);

    }
}

fn button_hovers(
    mut interaction_query: Query<(&Interaction, &mut BackgroundColor),(Changed<Interaction>, With<Button>)>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn populate_scoreboard(
    roster: Res<TeamRoster>,
    mut commands: Commands,
    scoreboard: Query<Entity, Added<ScoreboardUI>>,
    fonts: Res<Fonts>,
){
    let Ok(scoreboard_ui) = scoreboard.get_single() else {return}; // else spawn scoreboard?
    commands.entity(scoreboard_ui).despawn_descendants();
    for (team, players) in roster.teams.iter(){
        let mut color = Color::rgba(0.3, 0.15, 0.1, 0.95);
        if team == &TEAM_1{
            color = Color::rgba(0.15, 0.15, 0.2, 0.95);
        } 
        for player in players.iter(){
            println!("spawning");
            dbg!(player);
            commands.entity(scoreboard_ui).with_children(|parent| {
                parent.spawn(scoreboard_entry(color)).with_children(|parent| {
                    parent.spawn(plain_text(player.id.clone().to_string(), 14, &fonts));
                });
                parent.spawn(scoreboard_entry(color)).with_children(|parent| {
                    parent.spawn(plain_text("0 / 0 / 0", 14, &fonts)).insert(KDAText);
                });
            });
        }
    }
}

fn add_base_ui(
    mut commands: Commands,
    items: Res<Items>,
    fonts: Res<Fonts>,
    images: Res<Images>,
){
    commands.spawn(root_ui()).with_children(|parent| {
        // can be edited in HUD editor
        parent.spawn(header_holder()).with_children(|parent| {
            parent.spawn(editable_ui_wrapper()).with_children(|parent| {
                parent.spawn(header()).with_children(|parent| {
                    parent.spawn(timer_ui(&fonts));          
                });
            });
        });
        parent.spawn(killfeed_holder()).with_children(|parent| {
            parent.spawn(editable_ui_wrapper()).with_children(|parent| {
                parent.spawn(killfeed());
            });
        });
        parent.spawn(minimap_holder()).with_children(|parent| {
            parent.spawn(editable_ui_wrapper()).with_children(|parent| {
                parent.spawn(minimap(&images)).with_children(|parent| {
                    parent.spawn(minimap_arrow(&images));
                });
            });
        });
        parent.spawn(respawn_holder()).with_children(|parent| {
            parent.spawn(editable_ui_wrapper()).with_children(|parent| {
                parent.spawn(respawn_text(&fonts));
            });
        });
        parent.spawn(team_thumbs_holder()).with_children(|parent| {
            parent.spawn(editable_ui_wrapper()).with_children(|parent| {
                parent.spawn(team_thumbs());
            });
        });
        parent.spawn(bottom_left_ui_holder()).with_children(|parent| {
            parent.spawn(editable_ui_wrapper()).with_children(|parent| {
                parent.spawn(bottom_left_ui()).with_children(|parent| {
                    parent.spawn(stats_ui()).with_children(|parent| {
                        for x in 0..6{
                            parent.spawn(plain_text(format!("stat {}", x), 16, &fonts));
                        }
                    });
                    parent.spawn(build_and_kda()).with_children(|parent| {
                        parent.spawn(kda_ui()).with_children(|parent| {
                            parent.spawn(plain_text("0 / 0 / 0", 18, &fonts))
                                .insert(PersonalKDA);
                        });
                        parent.spawn(build_ui()).with_children(|parent| {
                            for _ in 0..3{
                                parent.spawn(build_slot()).with_children(|parent| {
                                    parent.spawn(item_image_build(&items, Item::Arondight));
                                });
                                parent.spawn(build_slot());
                            }
                        });
                    });
                });
            });
        });
        // non editable ui
        parent.spawn(tooltip());
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
            parent.spawn(list_categories()).with_children(|parent| {
                parent.spawn(category()).with_children(|parent| {
                    parent.spawn(category_text("Attack Damage", &fonts));
                });
                parent.spawn(category()).with_children(|parent| {
                    parent.spawn(category_text("Magical Power", &fonts));
                });
            });
            parent.spawn(list_items()).with_children(|parent| {
                parent.spawn(item_image(&items, Item::HiddenDagger));
                parent.spawn(item_image(&items, Item::Arondight));
                parent.spawn(item_image(&items, Item::SoulReaver));
                });
            parent.spawn(inspector()).with_children(|parent| {
                parent.spawn(gold_text(&fonts));
                parent.spawn(button()).with_children(|parent| {
                    parent.spawn(button_text("buy", &fonts));
                });            
            });
        });
    });
}


fn draggables(
    windows: Query<&mut Window, With<PrimaryWindow>>,
    // both queries can be the same entity or different
    handle_query: Query<(Entity, &Interaction, &Parent,), With<DragHandle>>,
    mut draggable_query: Query<(&mut Style, &Parent, &Node, &GlobalTransform, &Draggable)>,
    parent_query: Query<(&Node, &GlobalTransform)>,
    mut offset: Local<Vec2>,
    mut parent_offset: Local<Vec2>,
    mut max_offset: Local<Vec2>,
    mouse: Res<Input<MouseButton>>,
){
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };  
    if !mouse.pressed(MouseButton::Left) { return }
    for (handle_entity, interaction, handle_parent) in &handle_query {
        if *interaction != Interaction::Pressed{ 
            continue
        };
        for draggable in [handle_entity, handle_parent.get()]{
            let Ok((mut style, parent, node, gt, draggable)) = draggable_query.get_mut(draggable) else { 
                continue 
            };
            if mouse.just_pressed(MouseButton::Left){
                if let Ok((parent_node, parent_gt)) = parent_query.get(parent.get()){  
                    parent_offset.x = parent_gt.translation().x - parent_node.size().x/2.0;  
                    parent_offset.y = parent_gt.translation().y - parent_node.size().y/2.0;
                    *max_offset = parent_node.size() - node.size();
                }
                offset.x = cursor_pos.x - (gt.translation().x - node.size().x/2.0);
                offset.y = cursor_pos.y - (gt.translation().y - node.size().y/2.0);
            }   
            let mut left_position = cursor_pos.x - parent_offset.x - offset.x;
            let mut top_position = cursor_pos.y - parent_offset.y - offset.y;
            // clamp cant go outside bounds, with border if wanted
            if let Draggable::BoundByParent(border) = *draggable{
                let border = border as f32;
                left_position = left_position.clamp(0.0 - border, max_offset.x + border);
                top_position = top_position.clamp(0.0 - border, max_offset.y + border);
            };
            style.margin = UiRect::default();
            style.left = Val::Px(left_position);
            style.top = Val::Px(top_position);
            style.position_type = PositionType::Absolute;
        } 
    }
}

fn droppables(
    windows: Query<&mut Window, With<PrimaryWindow>>,
){
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };  

}

fn move_tooltip(
    mut tooltip: Query<&mut Style, With<Tooltip>>,
    mut move_events: EventReader<CursorMoved>,
){
    let Ok(mut style) = tooltip.get_single_mut() else{ return };
    if let Some(cursor_move) = move_events.iter().next(){        
        style.left = Val::Px(cursor_move.position.x);
        style.bottom = Val::Px(cursor_move.position.y);
    } 
}

fn hide_tooltip( // for when you close a menu when hovering, otherwise tooltip stays
    mut tooltip: Query<&mut Visibility, With<Tooltip>>,
){    
    let Ok(mut vis) = tooltip.get_single_mut() else{ return };
    *vis = Visibility::Hidden;   
}

fn load_tooltip(
    mut commands: Commands,
    mut tooltip: Query<(&mut Tooltip, &mut Visibility, Option<&Children>, Entity)>,
    hoverables: Query<(Entity, &AbilityTooltip, &Interaction), With<Hoverable>>,
    icons: Res<Icons>,
    fonts: Res<Fonts>,
){
    let Ok((mut tooltip, mut vis, children, tooltip_entity))
        = tooltip.get_single_mut() else{ return };
    let mut hovered_info: Option<AbilityTooltip> = None;
    for (hovering_entity, ability_info, interaction) in &hoverables{
        match interaction{
            Interaction::None => {
                if tooltip.0.is_some(){
                    tooltip.0 = None;
                }
            },
            Interaction::Hovered | Interaction::Pressed =>{
                if let Some(last_hovered) = tooltip.0{
                    if last_hovered == hovering_entity {
                        return
                    }
                }
                tooltip.0 = Some(hovering_entity.clone());
                hovered_info = Some(ability_info.clone());
            }
        }
    }
    match hovered_info {
        Some(info) =>{
            if let Some(children) = children{
                for child in children.iter(){
                    commands.entity(*child).despawn_recursive();
                }
            }
            let child = spawn_ability_tooltip(&mut commands, &icons, &fonts, &info.clone());
            commands.entity(tooltip_entity).add_child(child);
            *vis = Visibility::Visible;  
        }
        _ => {
            *vis = Visibility::Hidden;            
        }
    }

}


fn update_kda(
    mut kda_query: Query<&mut Text, With<PersonalKDA>>,
    mut scoreboard_kda_query: Query<&mut Text, (With<KDAText>, Without<PersonalKDA>)>,
    scoreboard: Res<Scoreboard>,
    mut death_events: EventReader<DeathEvent>,
    local_player: Res<Player>,
){
    if scoreboard.is_changed(){
        let Ok(mut kda_text) = kda_query.get_single_mut() else {return};
        for (player, kda) in scoreboard.kda_list.iter(){            
            if *player == *local_player {
                kda_text.sections[0].value = format!("{} / {} / {}", kda.kills, kda.deaths, kda.assists);
            }
        }
    }
}

pub fn killfeed_update(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    killfeed_query: Query<Entity, With<Killfeed>>,
){
    for death in death_events.iter(){
        let Ok(killfeed) = killfeed_query.get_single() else { return};
        let notification = commands.spawn(kill_notification()).id();
        commands.entity(killfeed).push_children(&[notification]);
    }
}

fn kill_notif_cleanup(
    mut commands: Commands,
    mut tween_events: EventReader<TweenCompleted>,
){
    for ev in tween_events.iter(){
        use TweenEvents::*;
        match TweenEvents::try_from(ev.user_data) {
            Ok(KillNotifEnded) => {commands.entity(ev.entity).despawn_recursive();}
            Err(_) | Ok(_) => (),
        }
    }
}



fn tick_despawn_timers(
    time: Res<Time>,
    mut things_to_despawn: Query<(Entity, &mut DespawnTimer)>,
    mut commands: Commands,
) {
    for (entity, mut timer) in &mut things_to_despawn {
        timer.0.tick(time.delta());
        if timer.0.finished(){
            commands.entity(entity).despawn_recursive();
        }
    }
}


fn tick_clock_ui(
    time: Res<Time>,
    mut clock: Query<&mut Text, With<InGameClock>>,
    game_details: Res<GameModeDetails>,
){    
    // Shouldnt do abs calculations every tick probably just 1/s, increment the seconds, increment minute if above 60
    let Ok(mut text) = clock.get_single_mut() else {
        return;
    };
    let elapsed = time.elapsed().as_secs() as i32;
    let adjusted = game_details.start_timer + elapsed;
    let mut sign = "";
    let minute = (adjusted  / 60).abs();
    let second = (adjusted % 60).abs();
    if adjusted < 0{
        sign = "-";
    } 

    text.sections[0].value = format!("{}{:02}:{:02}", sign, minute, second);
}


pub fn button_actions(
    mut interaction_query: Query<
        (&ButtonAction, &Interaction),
        (Changed<Interaction>, With<Button>),
    >,
    mut game_state_next: ResMut<NextState<GameState>>,
    mut ingamemenu_next: ResMut<NextState<InGameMenu>>,
    mut editing_hud_next: ResMut<NextState<EditingHUD>>,
    ingamemenu_state: Res<State<InGameMenu>>,
    editing_hud_state: Res<State<EditingHUD>>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut reset_ui_events: EventWriter<ResetUiEvent>,
) {
    for (button_action, interaction) in &mut interaction_query {
        if *interaction != Interaction::Pressed {continue};
        match button_action {
            ButtonAction::Play => {
                game_state_next.set(GameState::InGame);
            }
            ButtonAction::Settings => {

            }
            ButtonAction::Lobby => {
                game_state_next.set(GameState::MainMenu);
            }
            ButtonAction::Resume => {
                ingamemenu_next.set(ingamemenu_state.toggle());
            }
            ButtonAction::Exit => {
                app_exit_writer.send(AppExit);
            }
            ButtonAction::EditUi => {
                editing_hud_next.set(editing_hud_state.toggle());
                ingamemenu_next.set(InGameMenu::Closed);
            },
            ButtonAction::ResetUi => {
                reset_ui_events.send(ResetUiEvent);
            },
        }
    }
}

pub fn minimap_track(
    mut arrow_query: Query<&mut Style, With<MinimapPlayerIcon>>,
    trackables: Query<&GlobalTransform, With<Trackable>>
){
    let Ok(mut style) = arrow_query.get_single_mut() else { return };
    for tracking in trackables.iter(){
        let (player_left, player_top) =  (tracking.translation().x, tracking.translation().z);
        style.left = Val::Px(player_left);
        style.top = Val::Px(player_top);
    }
}

pub fn spawn_floating_health_bars(
    mut commands: Commands,
    health_bar_owners: Query<Entity, (With<Attributes>, Added<HasHealthBar>)>,
){
    for entity in &health_bar_owners{
        commands.spawn(follow_wrapper(entity)).insert(
            HealthBarHolder(entity)
        ).with_children(|parent| {
            parent.spawn(bar_background(12.0)).with_children(|parent| {
                parent.spawn(bar_fill(Color::rgba(0.94, 0.228, 0.128, 0.95))).insert((
                    HealthBar,
                    BarTrack{
                        entity: entity,
                        current: Stat::Health.into(),
                        max: Stat::HealthMax.into(),
                    }
                ));
            });
        });
    }
}

fn show_floating_health_bars(
    mut commands: Commands,
    possessing_query: Query<(&Transform, &Team)>,
    healthy: Query<(&Attributes, &Transform, &Team, Entity), With<HasHealthBar>>,
    mut health_bars: Query<(&mut Visibility, &HealthBarHolder)>,
    children_query: Query<&Children>,
    spectating: Res<Spectating>,
){
    let Ok((player_transform, team)) = possessing_query.get(spectating.0) else {return};
    for (attributes, target_transform, other_team, healthy_entity) in &healthy{
        let dir = (target_transform.translation - player_transform.translation).normalize();
        let direction_from_hp_bar = Quat::from_euler(EulerRot::XYZ, dir.x, dir.y, dir.z);
        for (mut vis, bar_holder) in &mut health_bars{
            if bar_holder.0 != healthy_entity { continue }
            if player_transform.rotation.dot(direction_from_hp_bar) > 0.0{
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}

#[derive(Component)]
pub struct BarTrack{
    pub entity: Entity,
    pub current: AttributeTag,
    pub max: AttributeTag,
}

pub fn bar_track(
    query: Query<&Attributes, Changed<Attributes>>,
    mut bar_query: Query<(&mut Style, &BarTrack)>,
){
    for (mut style, tracking) in &mut bar_query{
        let Ok(attributes) = query.get(tracking.entity) else {continue};
        let current = *attributes.get(&tracking.current).unwrap_or(&0.0);
        let max = *attributes.get(&tracking.max).unwrap_or(&100.0);
        let new_size = current / max;
        style.width = Val::Percent(new_size * 100.0);
    }
}

fn follow_in_3d(
    mut commands: Commands,
    mut follwer_query: Query<(&mut Style, &FollowIn3d, Entity)>,
    leader_query: Query<&GlobalTransform>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PlayerCam>>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };
    for (mut style, following, entity) in follwer_query.iter_mut() {
        let transform = if let Ok(gt) = leader_query.get(following.leader){
            gt.translation()
        } else if let Some(last_seen) = following.last_seen {
            last_seen.translation
        } else{
            commands.entity(entity).despawn_recursive();
            continue
        };

        let Some(viewport) = camera.world_to_viewport(camera_transform, transform + Vec3::Y * 2.0) else {
            continue;
        };
        style.left = Val::Px(viewport.x);
        style.top = Val::Px(viewport.y);
    }
}

#[derive(Component)]
pub struct Trackable;

#[derive(Component, PartialEq, Eq)]
pub enum ButtonAction{
    Play,
    Settings,
    Exit,
    Resume,
    Lobby,
    EditUi,
    ResetUi
}


pub mod main_menu;
pub mod ingame_menu;
pub mod mouse;
pub mod styles;
pub mod spectating;
pub mod hud_editor;
#[allow(unused_parens)]
pub mod ui_bundles;