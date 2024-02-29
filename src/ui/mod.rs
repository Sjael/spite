use bevy::{app::AppExit, prelude::*, window::PrimaryWindow};

use crate::{
    actor::{player::LocalPlayer, HasHealthBar},
    assets::{Fonts, Images},
    camera::PlayerCam,
    session::director::InGameSet,
    stats::{AttributeTag, Attributes, Stat},
    ui::{
        holding::{HoldingPlugin, Reposition},
        hud::HudPlugin,
        hud_editor::{EditingHUD, HudEditEvent, HudEditorPlugin},
        ingame_menu::*,
        main_menu::MainMenuPlugin,
        mouse::{build_mouse, MenuType, OpenMenus},
        scoreboard::ScoreboardPlugin,
        spectating::build_spectating,
        store::{ItemInspected, StoreEvent, StorePlugin, TransactionType, UndoPressEvent},
        styles::*,
        tooltip::TooltipPlugin,
        ui_bundles::*,
    },
    GameState,
};

pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            MainMenuPlugin,
            InGameMenuPlugin,
            HudEditorPlugin,
            HudPlugin,
            StorePlugin,
            TooltipPlugin,
            HoldingPlugin,
            ScoreboardPlugin,
        ));

        app.add_systems(Update, (button_hovers, button_actions));

        app.add_systems(
            Update,
            (
                reposition_with_bounds,
                spawn_floating_health_bars,
                tick_despawn_timers,
                follow_in_3d,
                bar_track,
                text_track,
                add_base_ui,
                //show_floating_health_bars.run_if(resource_exists::<Spectating>()),
            )
                .in_set(InGameSet::Update),
        );

        build_mouse(app);
        build_spectating(app);
    }
}

fn button_hovers(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>, Without<Category>),
    >,
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

fn add_base_ui(mut commands: Commands, fonts: Res<Fonts>, images: Res<Images>, query: Query<Entity, Added<PlayerCam>>) {
    let Ok(cam) = query.get_single() else { return };
    commands.spawn(root_ui(cam)).with_children(|parent| {
        // can be edited in HUD editor
        parent.spawn(header_holder()).with_children(|parent| {
            parent
                .spawn((editable_ui_wrapper(), EditableUI::Header))
                .with_children(|parent| {
                    parent.spawn(header()).with_children(|parent| {
                        parent.spawn(timer_ui(&fonts));
                    });
                });
        });
        parent.spawn(killfeed_holder()).with_children(|parent| {
            parent
                .spawn((editable_ui_wrapper(), EditableUI::Killfeed))
                .with_children(|parent| {
                    parent.spawn(killfeed());
                });
        });
        parent.spawn(minimap_holder()).with_children(|parent| {
            parent
                .spawn((editable_ui_wrapper(), EditableUI::Map))
                .with_children(|parent| {
                    parent.spawn(minimap(&images)).with_children(|parent| {
                        parent.spawn(minimap_arrow(&images));
                    });
                });
        });
        parent.spawn(team_thumbs_holder()).with_children(|parent| {
            parent
                .spawn((editable_ui_wrapper(), EditableUI::TeamThumbs))
                .with_children(|parent| {
                    parent.spawn(team_thumbs());
                });
        });
        // non editable ui
        parent.spawn(tooltip());
    });
}

#[derive(Component, Debug, PartialEq)]
pub struct BoundByParent(pub i32);

fn reposition_with_bounds(
    mut bound_query: Query<(
        &mut Style,
        &Node,
        &GlobalTransform,
        Option<&BoundByParent>,
        &mut Reposition,
        &Parent,
        Entity,
    )>,
    changed_children: Query<(), Changed<Children>>,
    parents: Query<&Node>,
    windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Some(cursor_pos) = window.cursor_position() else { return };
    for (mut style, node, gt, opt_bound, mut repos, parent, ent) in bound_query.iter_mut() {
        if repos.is_added() {
            continue
        }
        // skip everything that hasnt changed, or needed to recheck bounds since children will change size
        if !repos.is_changed() && changed_children.get(ent).is_err() {
            continue
        }
        let Vec2 { mut x, mut y } = repos.pos;
        // clamp cant go outside bounds, with border if wanted
        if let Some(BoundByParent(border)) = opt_bound {
            let Ok(parent_node) = parents.get(parent.get()) else { continue };
            // how far the draggable can move from their parent's top left and still be inside
            let max_bound = parent_node.size() - node.size();
            let border = *border as f32;
            let lower_bound = 0.0 - border;
            let upper_bound_x = max_bound.x + border;
            let upper_bound_y = max_bound.y + border;
            if x < lower_bound || x > upper_bound_x || y < lower_bound || y > upper_bound_y {
                // need to recalculate the inner mouse offset if getting bound, or going back will be wonky
                repos.inner_offset = cursor_pos - (gt.translation().xy() - node.size() / 2.0);
                x = x.clamp(lower_bound, upper_bound_x);
                y = y.clamp(lower_bound, upper_bound_y);
            }
        };
        style.left = Val::Px(x);
        style.top = Val::Px(y);
    }
}

fn tick_despawn_timers(
    time: Res<Time>,
    mut things_to_despawn: Query<(Entity, &mut DespawnTimer)>,
    mut commands: Commands,
) {
    for (entity, mut timer) in &mut things_to_despawn {
        timer.0.tick(time.delta());
        if timer.0.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}

pub fn button_actions(
    mut interaction_query: Query<(&ButtonAction, &Interaction), (Changed<Interaction>, With<Button>)>,
    mut game_state_next: ResMut<NextState<GameState>>,
    editing_hud_state: Res<State<EditingHUD>>,
    mut editing_hud_next: ResMut<NextState<EditingHUD>>,
    menu_state: Res<State<OpenMenus>>,
    mut menu_state_next: ResMut<NextState<OpenMenus>>,
    player: Option<Res<LocalPlayer>>,
    item_inspected: Res<ItemInspected>,
    mut app_exit_writer: EventWriter<AppExit>,
    mut reset_ui_events: EventWriter<HudEditEvent>,
    mut store_events: EventWriter<StoreEvent>,
    mut undo_events: EventWriter<UndoPressEvent>,
) {
    for (button_action, interaction) in &mut interaction_query {
        if *interaction != Interaction::Pressed {
            continue
        }
        match button_action {
            ButtonAction::Play => {
                game_state_next.set(GameState::InGame);
            }
            ButtonAction::Settings => {}
            ButtonAction::Lobby => {
                game_state_next.set(GameState::MainMenu);
            }
            ButtonAction::Resume => {
                menu_state_next.set(menu_state.toggle(MenuType::InGameMenu));
            }
            ButtonAction::Exit => {
                app_exit_writer.send(AppExit);
            }
            ButtonAction::EditUi => {
                editing_hud_next.set(editing_hud_state.toggle());
                menu_state_next.set(menu_state.toggle(MenuType::InGameMenu));
            }
            ButtonAction::ResetUi => {
                reset_ui_events.send(HudEditEvent::Reset);
            }
            ButtonAction::SaveUi => {
                editing_hud_next.set(editing_hud_state.toggle());
                reset_ui_events.send(HudEditEvent::Save);
            }
            ButtonAction::BuyItem => {
                let Some(player) = player.as_deref() else { continue };

                if let Some(inspected) = item_inspected.0.clone() {
                    store_events.send(StoreEvent {
                        player: **player,
                        item: inspected,
                        direction: TransactionType::Buy,
                        fresh: true,
                    });
                }
            }
            ButtonAction::SellItem => {
                let Some(player) = player.as_deref() else { continue };

                if let Some(inspected) = item_inspected.0.clone() {
                    store_events.send(StoreEvent {
                        player: **player,
                        item: inspected,
                        direction: TransactionType::Sell,
                        fresh: true,
                    });
                }
            }
            ButtonAction::UndoStore => {
                let Some(player) = player.as_deref() else { continue };
                undo_events.send(UndoPressEvent { entity: **player });
            }
            _ => (),
        }
    }
}

pub fn spawn_floating_health_bars(
    mut commands: Commands,
    health_bar_owners: Query<Entity, (With<Attributes>, Added<HasHealthBar>)>,
) {
    for entity in &health_bar_owners {
        commands
            .spawn(follow_wrapper(entity))
            .insert(HealthBarHolder(entity))
            .with_children(|parent| {
                parent.spawn(bar_background(12.0)).with_children(|parent| {
                    parent.spawn(bar_fill(Color::rgba(0.94, 0.228, 0.128, 0.95))).insert((
                        HealthBar,
                        BarTrack {
                            entity: entity,
                            current: Stat::Health.into(),
                            max: Stat::HealthMax.into(),
                        },
                    ));
                });
            });
    }
}

pub fn text_track(query: Query<&Attributes, Changed<Attributes>>, mut text_query: Query<(&mut Text, &TextTrack)>) {
    for (mut text, tracking) in &mut text_query {
        let Ok(attributes) = query.get(tracking.entity) else { continue };
        let mut whole_str = tracking.layout.clone();
        for stat in tracking.stat.iter() {
            let current = attributes.get(stat.clone());
            whole_str = whole_str.replacen("x", &current.trunc().to_string(), 1);
        }
        let Some(old_text) = text.sections.get(0) else { continue };
        *text = Text::from_section(whole_str, old_text.style.clone());
    }
}

pub fn bar_track(query: Query<&Attributes, Changed<Attributes>>, mut bar_query: Query<(&mut Style, &BarTrack)>) {
    for (mut style, tracking) in &mut bar_query {
        let Ok(attributes) = query.get(tracking.entity) else { continue };
        let current = attributes.get(tracking.current.clone());
        let max = attributes.get(tracking.max.clone());
        let new_size = current as f32 / max as f32;
        style.width = Val::Percent(new_size * 100.0);
    }
}

#[derive(Component)]
pub struct TextTrack {
    pub entity: Entity,
    pub stat: Vec<AttributeTag>,
    pub layout: String,
}
impl TextTrack {
    pub fn new(entity: Entity, stat: Stat) -> Self {
        let mut stats = vec![stat.clone().into()];
        let mut layout = "x".to_string();
        match stat {
            Stat::Health => {
                layout = "x / x  (+x)".to_string();
                stats.append(&mut vec![Stat::HealthMax.into(), Stat::HealthRegen.into()]);
            }
            Stat::CharacterResource => {
                layout = "x / x  (+x)".to_string();
                stats.append(&mut vec![
                    Stat::CharacterResourceMax.into(),
                    Stat::CharacterResourceRegen.into(),
                ]);
            }
            _ => (),
        }
        Self {
            entity,
            stat: stats,
            layout,
        }
    }
}

#[derive(Component)]
pub struct BarTrack {
    pub entity: Entity,
    pub current: AttributeTag,
    pub max: AttributeTag,
}

impl BarTrack {
    fn hp(entity: Entity) -> BarTrack {
        BarTrack {
            entity,
            current: Stat::Health.into(),
            max: Stat::HealthMax.into(),
        }
    }
    fn _res(entity: Entity) -> BarTrack {
        BarTrack {
            entity,
            current: Stat::CharacterResource.into(),
            max: Stat::CharacterResourceMax.into(),
        }
    }
}
/*
fn show_floating_health_bars(
    mut commands: Commands,
    query: Query<(&Transform, &Team)>,
    healthy: Query<(&Attributes, &Transform, &Team, Entity), With<HasHealthBar>>,
    mut health_bars: Query<(&mut Visibility, &HealthBarHolder)>,
    children_query: Query<&Children>,
    spectating: Res<Spectating>,
) {
    let Ok((player_transform, team)) = query.get(spectating.0) else {
        return
    };
    for (attributes, target_transform, other_team, healthy_entity) in &healthy {
        let dir = (target_transform.translation - player_transform.translation).normalize();
        let direction_from_hp_bar = Quat::from_euler(EulerRot::XYZ, dir.x, dir.y, dir.z);
        for (mut vis, bar_holder) in &mut health_bars {
            if bar_holder.0 != healthy_entity {
                continue
            }
            if player_transform.rotation.dot(direction_from_hp_bar) > 0.0 {
                *vis = Visibility::Visible;
            } else {
                *vis = Visibility::Hidden;
            }
        }
    }
}
*/

fn follow_in_3d(
    mut commands: Commands,
    mut follwer_query: Query<(&mut Style, &FollowIn3d, Entity)>,
    leader_query: Query<&GlobalTransform>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PlayerCam>>,
) {
    let Ok((camera, camera_transform)) = camera_query.get_single() else { return };
    for (mut style, following, entity) in follwer_query.iter_mut() {
        let transform = if let Ok(gt) = leader_query.get(following.leader) {
            gt.translation()
        } else if let Some(last_seen) = following.last_seen {
            last_seen.translation
        } else {
            commands.entity(entity).despawn_recursive();
            continue
        };

        let Some(viewport) = camera.world_to_viewport(camera_transform, transform + Vec3::Y * 2.0) else { continue };
        style.left = Val::Px(viewport.x);
        style.top = Val::Px(viewport.y);
    }
}

#[derive(Component, PartialEq, Eq)]
pub enum ButtonAction {
    Play,
    Settings,
    Exit,
    Resume,
    Lobby,
    EditUi,
    ResetUi,
    SaveUi,
    ClearFilter,
    BuyItem,
    SellItem,
    UndoStore,
}

pub mod holding;
pub mod hud;
pub mod hud_editor;
pub mod ingame_menu;
pub mod main_menu;
pub mod mouse;
pub mod scoreboard;
pub mod spectating;
pub mod store;
pub mod styles;
pub mod tooltip;
#[allow(unused_parens)]
pub mod ui_bundles;
