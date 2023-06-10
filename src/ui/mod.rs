use bevy::{prelude::*, window::PrimaryWindow, app::AppExit};
use bevy_tweening::TweenCompleted;

use crate::{
    ui::{ui_bundles::*,styles::*, player_ui::*, mouse::*, ingame_menu::*, main_menu::*,},
    player::{PlayerCam, IncomingDamageLog, OutgoingDamageLog},  
    ability::{AbilityInfo, ability_bundles::FloatingDamage, DamageInstance},
    game_manager::{GameModeDetails, DeathEvent}, assets::{Icons, Items, Fonts, Images}, GameState, item::Item, 
    
};


pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<MouseState>();
        app.add_state::<TabMenuOpen>();
        app.add_event::<MenuEvent>();

        app.add_event::<BuyItem>();


        app.add_systems((
            spawn_main_menu, 
        ).in_schedule(OnEnter(GameState::MainMenu)));
        app.add_systems((
            exit_game_main_menu,
        ).in_set(OnUpdate(GameState::MainMenu)));
        app.add_systems((
            cleanup, 
        ).in_schedule(OnExit(GameState::MainMenu)));

        app.add_systems((
            add_player_ui,
            add_ability_icons,
            move_tooltip,
            update_cooldowns,
            spawn_floating_damage,
            follow_in_3d,
            update_health,
            update_character_resource,
            tick_clock_ui,
            drag_store, // combine these 2 later
            drag_items,
            free_mouse,
            mouse_with_free_key,
            //mouse_menu_open,
            menu_toggle,
            toggle_ingame_menu,
        ).in_set(OnUpdate(GameState::InGame)));
        app.add_systems((            
            update_damage_log,
            killfeed_update,
            kill_notif_cleanup,
        ).in_set(OnUpdate(GameState::InGame)));

        app.add_systems((
            add_base_ui,
            add_ingame_menu,
        ).in_schedule(OnEnter(GameState::InGame)));
        app.add_systems((
            load_tooltip.run_if(in_state(MouseState::Free)).in_set(OnUpdate(GameState::InGame)),
            hide_tooltip.in_schedule(OnExit(MouseState::Free)).in_set(OnUpdate(GameState::InGame)),
        ));
        app.add_systems((
            button_hovers,
            button_actions,
            tick_despawn_timers,
        ));

    }
}
#[derive(Component, Debug, Clone, Default)]
pub struct ItemId{
    id: u32,
    //stats: Vec<Attribute>,
    //passive: ItemPassive,
}

impl ItemId{
    pub fn new(id: u32) -> Self{
        Self{
            id,
            //stats: HashMap::default(),
        }
    }

    pub fn id(&mut self) -> u32{
        self.id
    }
}
pub struct BuyItem (ItemId);
fn button_hovers(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    //mut text_query: Query<&mut Text>,
    mut buy_events: EventWriter<BuyItem>,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                *color = PRESSED_BUTTON.into();
                buy_events.send(BuyItem(ItemId::new(31)))
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

fn add_base_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    items: Res<Items>,
    fonts: Res<Fonts>,
    images: Res<Images>,
){
    commands.spawn(header()).with_children(|parent| {
        parent.spawn(timer_ui(&fonts));          
    });
    commands.spawn(killfeed());
    commands.spawn(minimap(&images));
    commands.spawn(respawn_text(&fonts));
    commands.spawn(tooltip());
    commands.spawn(tab_panel()).with_children(|parent| {
        parent.spawn(damage_log()).with_children(|parent| {
            parent.spawn(log_outgoing());
            parent.spawn(log_incoming());
        });
        parent.spawn(scoreboard());
        parent.spawn(death_recap());
        parent.spawn(abilities_panel());
    });
    commands.spawn(team_thumbs());
    commands.spawn(bottom_left_ui()).with_children(|parent| {
        parent.spawn(stats_ui());
        parent.spawn(build_and_kda()).with_children(|parent| {
            parent.spawn(kda_ui());
            parent.spawn(build_ui());
        });
    });
    commands.spawn(store()).with_children(|parent| {
        parent.spawn(drag_bar());
        parent.spawn(list_categories()).with_children(|parent| {
            parent.spawn(category()).with_children(|parent| {
                parent.spawn(category_text("Attack Damage".to_owned(),&fonts));
            });
            parent.spawn(category()).with_children(|parent| {
                parent.spawn(category_text("Magical Power".to_owned(),&fonts));
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
                parent.spawn(button_text("buy".to_string(),&fonts));
            });            
        });
    });
    // Store 
}

fn drag_items( // need to consume click event so you dont drag multiple things
    windows: Query<&mut Window, With<PrimaryWindow>>,
    mut draggable_query: Query<(&Interaction, &Parent, &mut Style, &Node, &GlobalTransform), With<Draggable>>,
    parent_query: Query<(&Node, &GlobalTransform)>,
    mut offset: Local<Vec2>,
    mut parent_offset: Local<Vec2>,
    mouse: Res<Input<MouseButton>>,
    mouse_is_free: Res<State<MouseState>>,
){    
    let Ok(window) = windows.get_single() else { return };
    for (interaction, parent, mut style, node, gt) in &mut draggable_query {
        if *interaction == Interaction::Clicked && mouse_is_free.0 == MouseState::Free{ 
            if let Some(cursor_pos) = window.cursor_position() {   
                // cursor is from bottom left, ui is from top left so we need to flip  
                let cursor_y_flip = window.height() - cursor_pos.y; 
                if mouse.just_pressed(MouseButton::Left){
                    if let Ok((parent_node, parent_gt)) = parent_query.get(parent.get()){  
                        parent_offset.x = parent_gt.translation().x - parent_node.size().x/2.0;  
                        parent_offset.y = parent_gt.translation().y - parent_node.size().y/2.0;
                    }
                    // save cursor_pos when clicked
                    offset.x = cursor_pos.x - gt.translation().x + node.size().x/2.0;
                    offset.y = cursor_y_flip - gt.translation().y + node.size().y/2.0;
                } 
                style.position.left = Val::Px(cursor_pos.x - parent_offset.x - offset.x);
                style.position.top = Val::Px(cursor_y_flip - parent_offset.y - offset.y);
                style.position_type = PositionType::Absolute;
            }
        }
    }
}

fn drag_store(
    windows: Query<&mut Window, With<PrimaryWindow>>,
    handle_query: Query<(Entity, &Interaction, &Parent), With<DragStore>>,
    // both queries can be the same entity or different
    mut draggable_query: Query<(&mut Style, &Node, &GlobalTransform), With<StoreMain>>,
    mut offset: Local<Vec2>,
    mouse: Res<Input<MouseButton>>,
    mouse_is_free: Res<State<MouseState>>,
){
    let Ok(window) = windows.get_single() else { return };
    for (_e, interaction, handle_parent) in &handle_query {
        if *interaction == Interaction::Clicked && mouse_is_free.0 == MouseState::Free{ 
            if let Ok((mut style, node, gt)) = draggable_query.get_mut(handle_parent.get()){  
                if let Some(cursor_pos) = window.cursor_position() {   
                    // cursor is from bottom left, ui is from top left so we need to flip  
                    let cursor_y_flip = window.height() - cursor_pos.y; 
                    if mouse.just_pressed(MouseButton::Left){
                        offset.x = cursor_pos.x - (gt.translation().x - node.size().x/2.0);
                        offset.y = cursor_y_flip - (gt.translation().y - node.size().y/2.0);
                    }   
                    style.position.left = Val::Px(cursor_pos.x - offset.x);
                    style.position.top = Val::Px(cursor_y_flip - offset.y);
                }
            }
        }
    }
}





fn move_tooltip(
    mut tooltip: Query<&mut Style, With<Tooltip>>,
    mut move_events: EventReader<CursorMoved>,
){
    let Ok(mut style) = tooltip.get_single_mut() else{ return };
    if let Some(cursor_move) = move_events.iter().next(){        
        style.position.left = Val::Px(cursor_move.position.x);
        style.position.bottom = Val::Px(cursor_move.position.y);
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
    hoverables: Query<(Entity, &AbilityInfo, &Interaction), With<Hoverable>>,
    asset_server: Res<AssetServer>,
    icons: Res<Icons>,
    fonts: Res<Fonts>,
){
    let Ok((mut tooltip, mut vis, children, tooltip_entity))
        = tooltip.get_single_mut() else{ return };
    let mut hovered_info: Option<AbilityInfo> = None;
    for (hovering_entity, ability_info, interaction) in &hoverables{
        match interaction{
            Interaction::None => {
                if tooltip.0.is_some(){
                    tooltip.0 = None;
                }
            },
            Interaction::Hovered | Interaction::Clicked =>{
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



pub fn killfeed_update(
    mut commands: Commands,
    mut death_events: EventReader<DeathEvent>,
    killfeed_query: Query<Entity, With<Killfeed>>,
    asset_server: Res<AssetServer>,
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
            Err(err) => warn!(err),
        }
    }
}


fn tick_despawn_timers(
    time: Res<Time>,
    mut things_to_despawn: Query<(Entity, &mut DespawnTimer)>,
    mut commands: Commands,
) {
    for (entity, mut timer) in &mut things_to_despawn {
        // remove if finished
        timer.0.tick(time.delta());
        if timer.0.finished(){
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn update_damage_log(
    incoming_logs: Query<&IncomingDamageLog>,
    outgoing_logs: Query<&OutgoingDamageLog>,
    incoming_ui: Query<(Entity, Option<&Children>), With<IncomingLogUi>>,
    outgoing_ui: Query<(Entity, Option<&Children>), With<OutgoingLogUi>>,
    mut commands: Commands,
    fonts: Res<Fonts>,
    mut damage_events: EventReader<DamageInstance>,
){
    let Ok((incoming_log_entity, incoming_children)) = incoming_ui.get_single() else {return};
    let Ok((outgoing_log_entity, outgoing_children)) = outgoing_ui.get_single() else {return};
    
    for damage_instance in damage_events.iter(){
        if let Ok(attacker_log) = outgoing_logs.get(damage_instance.attacker) {

        }
        commands.entity(incoming_log_entity).with_children(|parent| {
            parent.spawn(damage_entry(damage_instance.damage.clone().to_string(), &fonts));
        });
        commands.entity(outgoing_log_entity).with_children(|parent| {
            parent.spawn(damage_entry(damage_instance.damage.clone().to_string(), &fonts));
        });
        if let Ok(defender_log) = incoming_logs.get(damage_instance.defender) {
            
        }
    } 

}
 
fn spawn_floating_damage(
    mut commands: Commands,
    query: Query<(Entity, &FloatingDamage), Added<FloatingDamage>>,
    asset_server: Res<AssetServer>,
    fonts: Res<Fonts>,
) {
    for (entity, damage) in query.iter() {
        commands
            .spawn(follow_wrapper(entity)).with_children(|parent| {            
                parent.spawn(follow_inner_text(damage.0.to_string(), &fonts));
            });
    }
}


fn follow_in_3d(
    mut commands: Commands,
    mut query: Query<(&mut Style, &FollowIn3d, Entity)>,
    world_query: Query<&GlobalTransform>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PlayerCam>>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else {
        return;
    };


    for (mut style, follow, entity) in query.iter_mut() {
        // the ability got removed, need to change this to just despawn after timer
        let Ok(world) = world_query.get(follow.0) else {
            commands.entity(entity).despawn_recursive();
            continue
        };

        let Some(viewport) = camera.world_to_viewport(camera_transform, world.translation() + Vec3::Y * 2.0) else {
            continue;
        };

        style.position = UiRect {
            left: Val::Px(viewport.x),
            bottom: Val::Px(viewport.y),
            ..default()
        };
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
        (&ButtonAction, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut kb: ResMut<Input<KeyCode>>,
) {
    for (button_action, interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                match button_action {
                    ButtonAction::Play => {
                        next_state.set(GameState::InGame);
                    }
                    ButtonAction::Settings => {

                    }
                    ButtonAction::Lobby => {
                        next_state.set(GameState::MainMenu);
                    }
                    ButtonAction::Resume => {
                        kb.press(KeyCode::Escape);
                    }
                    ButtonAction::Exit => {
                        app_exit_writer.send(AppExit);
                    }
                }
            }
            Interaction::Hovered => {
            }
            Interaction::None => {
            }
        }
    }
}


#[derive(Component, PartialEq, Eq)]
pub enum ButtonAction{
    Play,
    Settings,
    Exit,
    Resume,
    Lobby,
}


pub mod main_menu;
pub mod ingame_menu;
pub mod mouse;
pub mod styles;
pub mod player_ui;
#[allow(unused_parens)]
pub mod ui_bundles;