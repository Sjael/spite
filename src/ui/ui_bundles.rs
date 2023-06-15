use std::time::Duration;

use bevy::{prelude::*, ui::FocusPolicy};
use bevy_tweening::{Animator,  lens::{ UiPositionLens, UiBackgroundColorLens}, EaseFunction, Tween, Delay};

use crate::{ability::{AbilityInfo, Ability}, assets::{Icons, Items, Fonts, Images}, item::Item};

//
// Player UI Components
//
// We impl Bundle so that we can chain any components together

#[derive(Component, Debug)]
pub struct Tooltip(pub Option<Entity>);
impl Default for Tooltip{
    fn default() -> Self {
        Tooltip(None)
    }
}


#[derive(Component)]
pub struct Hoverable;

pub struct HoverEvent{
    entity: Entity,
    // info
}


#[derive(Component, Debug)]
pub struct HealthBarText;

#[derive(Component, Debug)]
pub struct HealthBar;

#[derive(Component, Debug)]
pub struct ResourceBar;

#[derive(Component, Debug)]
pub struct ResourceBarText;

#[derive(Component, Debug)]
pub struct CooldownIconText;

#[derive(Component, Debug)]
pub struct InGameClock;
#[derive(Component, Debug)]
pub struct RespawnText;

pub fn respawn_text(fonts: &Res<Fonts>) -> impl Bundle{(
    TextBundle {
        style: Style {
            margin: UiRect {
                left: Val::Auto,
                right:Val::Auto,
                ..default()
            },
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Percent(10.),
                left: Val::Percent(-20.),
                ..default()
            },
            ..default()
        },
        text: Text::from_sections([
            TextSection {
                value: "Respawning in\n".to_string(),
                style: TextStyle {
                    font: fonts.exo_light.clone(),
                    font_size: 18.0,
                    color: Color::YELLOW,
                },
            },
            TextSection {
                value: "12".to_string(),
                style: TextStyle {
                    font: fonts.exo_bold.clone(),
                    font_size: 36.0,
                    color: Color::WHITE,
                },
            },
        ]).with_alignment(TextAlignment::Center),
        visibility: Visibility::Hidden,
        background_color: Color::rgba(0.8, 0.7, 0.2, 0.4).into(),
        ..default()
    },
    RespawnText
)}

#[derive(Component)]
pub struct Minimap;

pub fn minimap(images: &Res<Images>) -> impl Bundle {(
    ImageBundle {
        style: Style {
            max_size: Size::new(Val::Px(300.), Val::Px(300.)),
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Percent(10.),
                right: Val::Percent(10.),
                ..default()
            },
            ..default()
        },
        background_color: Color::rgba(0.8, 1., 0.8, 1.0).into(),
        image: images.minimap.clone().into(),
        ..default()
    },
    Interaction::None,
    Minimap
)}

#[derive(Component)]
pub struct Killfeed;

pub fn killfeed() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(150.), Val::Percent(50.)),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            margin: UiRect {
                left: Val::Auto,
                top: Val::Auto,
                bottom: Val::Auto,
                right: Val::Px(30.)
            },
            ..default()
        },
        background_color: Color::rgba(0.9, 0.2, 0.2, 0.4).into(),
        ..default()
    },
    Killfeed,
    Name::new("Killfeed"),
)}

#[derive(Component)]
pub struct KillNotification;
pub fn kill_notification() -> impl Bundle{
    const ENEMY_COLOR: Color = Color::rgb(0.94, 0.1, 0.2);
    const ALLY_COLOR: Color = Color::rgb(0.3, 0.1, 0.94);
    const NEUTRAL_COLOR: Color = Color::rgb(0.7, 0.7, 0.2);
    let killer_on_team = false;
    let mut selected_color : Color;
    if killer_on_team {
        selected_color = ALLY_COLOR.clone();
    } else{
        selected_color = ENEMY_COLOR.clone();
    }
    let killfeed_offset = 200.;
    let delay_seconds = 8;
    let tween_pos = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(500),
        UiPositionLens {
            start: UiRect{
                right:Val::Px(killfeed_offset),
                ..default()
            },
            end: UiRect{
                right:Val::Px(0.),
                ..default()
            },
        },
    );
    let tween_opac_in = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(500),
        UiBackgroundColorLens {
            start: *selected_color.set_a(0.0),
            end: *selected_color.set_a(0.9),
        },
    );
    let tween_opac_out = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_secs(1),
        UiBackgroundColorLens {
            start: *selected_color.set_a(0.9),
            end: *selected_color.set_a(0.0),
        },
    ).with_completed_event(
        TweenEvents::KillNotifEnded as u64,
    );

    (
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Px(64.)),
            ..default()
        },
        background_color: Color::rgba(0.2, 0.2, 1.0, 0.1).into(),
        ..default()
    },
    KillNotification,
    Name::new("Kill Notification"),
    Animator::new(tween_pos),
    Animator::new(tween_opac_in
        .then(Delay::new(
            Duration::from_secs(delay_seconds),
        ))
        .then(tween_opac_out)
    ),
)}

#[derive(Clone, Copy)]
pub enum TweenEvents{
    KillNotifEnded = 0,
}

impl TryFrom<u64> for TweenEvents{
    type Error = String;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        use TweenEvents::*;
        match value{
            0 => Ok(KillNotifEnded),
            _ => Err("invalid TweenEvents index".to_string()),
        }
    }
}

pub fn bottom_left_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(220.), Val::Px(140.)),
            position_type: PositionType::Absolute,
            position: UiRect {
                bottom: Val::Px(40.),
                left: Val::Px(40.),
                ..default()
            },
            ..default()
        },
        ..default()
    },
    Name::new("Bottomleft"),
)}
pub fn stats_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(60.), Val::Percent(100.)),
            flex_direction:FlexDirection::Column,
            ..default()
        },
        background_color: Color::rgba(0.9, 0.1, 0.9, 0.2).into(),
        ..default()
    },
    Name::new("Stats"),
)}
pub fn build_and_kda() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            flex_direction:FlexDirection::Column,
            ..default()
        },
        ..default()
    },
    Name::new("Build and KDA"),
)}
#[derive(Component)]
pub struct BuildUI;
pub fn build_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            ..default()
        },
        background_color: Color::rgba(0.9, 0.6, 0.1, 0.2).into(),
        ..default()
    },
    BuildUI,
    Name::new("BuildUI"),
)}
pub fn kda_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Px(40.)),
            ..default()
        },
        background_color: Color::rgba(0.1, 0.6, 0.9, 0.2).into(),
        ..default()
    },
    Name::new("KDA"),
)}

#[derive(Component)]
pub struct TeammateThumbs;

pub fn team_thumbs() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(200.), Val::Px(80.)),
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(40.),
                left: Val::Px(40.),
                ..default()
            },
            ..default()
        },
        background_color: Color::rgba(0.9, 0.9, 0.2, 0.4).into(),
        ..default()
    },
    TeammateThumbs,
    Name::new("Team thumbs"),
)}

#[derive(Component)]
pub struct HeaderUI;

pub fn header() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(35.), Val::Px(80.)),
            position_type: PositionType::Absolute,
            justify_content:JustifyContent::Center,
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            ..default()
        },
        background_color: Color::rgba(0.1, 0.2, 0.7, 0.4).into(),
        ..default()
    },
    HeaderUI,
    Name::new("Header UI"),
)}

pub fn timer_ui(fonts: &Res<Fonts>) -> impl Bundle {(
    TextBundle {
        style: Style {
            margin: UiRect {
                top: Val::Px(30.),
                ..default()
            },
            ..default()
        },
        text: Text::from_section(
            "14:30",
            TextStyle {
                font: fonts.exo_bold.clone(),
                font_size: 16.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    },
    InGameClock,
)}

#[derive(Component)]
pub struct RootUI;

pub fn root_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            ..default()
        },
        ..default()
    },
    RootUI,
    Name::new("UI"),
)}

#[derive(Component)]
pub struct PlayerUI;
pub fn player_bottom_container() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(25.), Val::Auto),
            position_type: PositionType::Absolute,
            position: UiRect{
                bottom: Val::Px(0.),
                ..default()
            },
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            flex_direction: FlexDirection::Column,
            align_items:AlignItems::Center,
            ..default()
        },
        background_color: Color::rgba(0.1, 0.7, 0.7, 0.4).into(),
        ..default()
    },
    PlayerUI,
)}

pub fn effect_bar() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Px(60.0)),
            margin: UiRect {
                bottom: Val::Px(30.),
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            ..default()
        },
        ..default()
    }
)}

pub fn buff_bar() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(50.), Val::Percent(100.)),
            ..default()
        },
        background_color: Color::rgba(0.1, 0.1, 1., 0.4).into(),
        ..default()
    }
)}

pub fn debuff_bar() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(50.), Val::Percent(100.)),
            ..default()
        },
        background_color: Color::rgba(1., 0.1, 0.2, 0.4).into(),
        ..default()
    }
)}

pub fn bar_wrapper() -> impl Bundle{(
    NodeBundle{
        style: Style{            
            size: Size::new(Val::Percent(100.0), Val::Auto),
            flex_direction: FlexDirection::Column,
            margin: UiRect {
                bottom: Val::Px(10.),
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            ..default()
        },
        ..default()
    }
)}

pub fn hp_bar(height: f32) -> impl Bundle {(
    NodeBundle {
        style: Style {
            align_self: AlignSelf::FlexStart,
            align_items: AlignItems::FlexStart,
            justify_content: JustifyContent::FlexStart,
            size: Size::new(Val::Percent(100.0), Val::Px(height)),
            margin: UiRect {
                bottom: Val::Px(3.),
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            ..default()
        },
        background_color: Color::rgba(0.1, 0.1, 0.2, 0.1).into(),
        ..default()
    }
)}

pub fn hp_bar_color() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(60.0), Val::Percent(100.0)),
            ..default()
        },
        background_color: Color::rgb(0.27, 0.77, 0.26).into(),
        ..default()
    },
    HealthBar,
)}
pub fn resource_bar_color() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(60.0), Val::Percent(100.0)),
            ..default()
        },
        background_color: Color::rgb(0.88, 0.67, 0.01).into(),
        ..default()
    },
    ResourceBar,
)}

pub fn hp_bar_inner() -> impl Bundle{(    
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    },
)}

pub fn hp_bar_text(fonts: &Res<Fonts>) -> impl Bundle {(
    TextBundle {
        style: Style {
            margin:UiRect::top(Val::Px(-1.)),
            ..default()
        },
        text: Text::from_section(
            "200",
            TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: 18.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    },
    HealthBarText,
)}

pub fn resource_bar_text(fonts: &Res<Fonts>) -> impl Bundle {(
    TextBundle {
        style: Style {
            margin:UiRect::top(Val::Px(-2.)),
            ..default()
        },
        text: Text::from_section(
            "100",
            TextStyle {
                font: fonts.exo_regular.clone(),
                font_size: 14.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    },
    ResourceBarText,
)}

#[derive(Component, Debug)]
pub struct AbilityHolder;

pub fn ability_holder() -> impl Bundle {(
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::FlexStart,
            margin: UiRect {
                bottom: Val::Px(30.),
                ..default()
            },
            gap:Size::new(Val::Px(10.), Val::Auto),
            ..default()
        },
        ..default()
    },
    AbilityHolder,
    Name::new("Ability Holder"),
)}

pub fn ability_image(icons: &Res<Icons>, ability: Ability) -> impl Bundle {(
    ImageBundle {
        style: Style {
            size: Size::new(Val::Px(48.), Val::Px(48.)),
            ..default()
        },
        image: match ability{
            Ability::Frostbolt => icons.frostbolt.clone().into(),
            Ability::Fireball => icons.fireball.clone().into(),
            Ability::Dash => icons.dash.clone().into(),
            _ => icons.basic_attack.clone().into(),
        },
        ..default()
    },
    Interaction::None,
    Name::new("Ability Image"),
)}

pub fn cd_text(fonts: &Res<Fonts>) -> impl Bundle {(
    TextBundle {
        style: Style {
            margin: UiRect::all(Val::Auto),
            position_type: PositionType::Absolute,
            position: UiRect {
                ..default()
            },
            ..default()
        },
        text: Text::from_section(
            "",
            TextStyle {
                font: fonts.exo_bold.clone(),
                font_size: 30.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    },
    CooldownIconText,
    Name::new("Cooldown Text"),
)}

pub fn tooltip() -> impl Bundle{(
    NodeBundle{
        style:Style {
            position_type: PositionType::Absolute,
            min_size: Size::new(Val::Px(150.), Val::Px(100.)), // only cus adding to an empty makes it weird 1 frame
            ..default()
        },
        background_color: Color::NONE.into(),
        z_index: ZIndex::Global(2),
        ..default()
    },
    Tooltip::default(),
    Name::new("Tooltip"),
)}

pub fn spawn_ability_tooltip(
    commands: &mut Commands, 
    icons: &Res<Icons>,
    fonts: &Res<Fonts>,
    info: &AbilityInfo,
) -> Entity{
    commands.spawn(tooltip_bg()).with_children(|parent| {
        parent.spawn(tooltip_desc(&fonts, info.description.clone()));
        parent.spawn(tooltip_title(&fonts, info.title.clone()));
        parent.spawn(tooltip_image(&icons, info.title.clone()));
    }).id() 
}

pub fn tooltip_bg() -> impl Bundle {(
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Baseline,
            margin: UiRect {
                bottom: Val::Px(10.),
                ..default()
            },
            padding: UiRect::all(Val::Px(20.)),
            ..default()
        },
        background_color: Color::rgba(0., 0., 0., 1.0).into(),
        ..default()
    },
    Name::new("Background Tooltip"),
)}

pub fn tooltip_title(fonts: &Res<Fonts>, text: String) -> impl Bundle {(
    TextBundle {
        style: Style {
            position: UiRect {
                ..default()
            },
            ..default()
        },
        text: Text::from_section(
            text,
            TextStyle {
                font: fonts.exo_light.clone(),
                font_size: 30.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    },
    TooltipContents::Title,
)}

pub fn tooltip_desc(fonts: &Res<Fonts>, text: String) -> impl Bundle {(
    TextBundle {
        style: Style {
            margin: UiRect {
                top: Val::Px(15.),
                left: Val::Px(0.),
                ..default()
            },
            position: UiRect {
                ..default()
            },
            max_size: Size::new(Val::Px(320.), Val::Auto),
            ..default()
        },
        text: Text::from_section(
            text,
            TextStyle {
                font: fonts.exo_light.clone(),
                font_size: 16.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    },
    TooltipContents::Description,
)}

#[derive(Component, Debug)]
pub enum TooltipContents {
    Title,
    Description,
    Image,
}

pub fn tooltip_image(icons: &Res<Icons>, path: String) -> impl Bundle {(
    ImageBundle {
        style: Style {
            size: Size::new(Val::Px(64.), Val::Px(64.)),
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Px(-74.),
                top: Val::Px(0.),
                ..default()
            },
            ..default()
        },
        image: match path.to_lowercase().as_str(){
            "frostbolt" => icons.frostbolt.clone().into(),
            "dash" => icons.dash.clone().into(),
            "fireball" => icons.fireball.clone().into(),
            _ => icons.basic_attack.clone().into(),
        },
        ..default()
    },
    TooltipContents::Image,
)}



#[derive(Component, Debug)]
pub struct InGameMenu;
pub fn ingame_menu() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(20.), Val::Percent(50.)),
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        background_color: Color::rgba(0.2, 0.2, 0.2, 0.4).into(),
        z_index: ZIndex::Global(10),
        visibility: Visibility::Hidden,
        ..default()
    },
    InGameMenu,
    Name::new("InGame Menu"),
)}

pub fn ingame_menu_button() -> impl Bundle {(
    ButtonBundle {
        style: Style {
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            flex_grow: 1.0,
            ..default()
        },
        background_color: Color::rgb(0.15, 0.15, 0.15).into(),
        ..default()
    },
    HoverButton,    
)}

pub fn ingame_menu_button_text(text: String, fonts: &Res<Fonts>) -> impl Bundle {(
    TextBundle::from_section(
        text.to_owned(),
        TextStyle {
            font: fonts.exo_regular.clone(),
            font_size: 24.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    )
)}

#[derive(Component)]
pub struct DamageLog;
#[derive(Component)]
pub struct TabPanel;

pub fn tab_panel() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(70.), Val::Percent(70.)),
            position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        visibility: Visibility::Hidden,
        z_index: ZIndex::Global(8),
        ..default()
    },
    TabPanel,
    Name::new("TabPanel"),
)}

#[derive(Component)]
pub struct TabMenuWrapper(pub TabMenuType);

#[derive(Default, Eq, PartialEq)]
pub enum TabMenuType{
    Scoreboard,
    DamageLog,
    DeathRecap,
    Abilities,
    #[default]
    None,
}
pub fn damage_log() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            position_type: PositionType::Absolute,
            ..default()
        },
        ..default()
    },
    TabMenuWrapper(TabMenuType::DamageLog),
    DamageLog,
)}

#[derive(Component)]
pub struct OutgoingLogUi;
#[derive(Component)]
pub struct IncomingLogUi;

pub fn log_outgoing() -> impl Bundle {(    
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            ..default()
        },
        background_color: Color::rgba(0.14, 0.14, 0.3, 0.99).into(),
        ..default()
    },
    OutgoingLogUi,
)}
pub fn log_incoming() -> impl Bundle {(    
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            flex_grow: 1.0,
            ..default()
        },
        background_color: Color::rgba(0.3, 0.14, 0.1, 0.99).into(),
        ..default()
    },
    IncomingLogUi,
)}

pub fn damage_entry(text: String, fonts: &Res<Fonts>,) -> impl Bundle {(
    TextBundle {
        style: Style {
            margin : UiRect{
                top: Val::Px(12.),
                ..default()
            },
            ..default()
        },
        text: Text::from_section(
            text,
            TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: 20.0,
                color: Color::YELLOW,
            },
        ),
        ..default()
    },
    Name::new("Damage Entry"),
    DespawnTimer(
        Timer::new(Duration::from_secs(30), TimerMode::Once)
    )
)}

#[derive(Component, Debug)]
pub struct DespawnTimer(pub Timer);

pub fn scoreboard() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(300.), Val::Px(200.)),
            position_type: PositionType::Absolute,
            ..default()
        },
        background_color: Color::rgba(0.1, 0.8, 0.5, 0.4).into(),
        ..default()
    },
    TabMenuWrapper(TabMenuType::Scoreboard),
)}
pub fn abilities_panel() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(300.), Val::Px(200.)),
            position_type: PositionType::Absolute,
            ..default()
        },
        background_color: Color::rgba(0.8, 0.8, 0.5, 0.4).into(),
        ..default()
    },
    TabMenuWrapper(TabMenuType::Abilities),
)}
pub fn death_recap() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(300.), Val::Px(200.)),
            position_type: PositionType::Absolute,
            ..default()
        },
        background_color: Color::rgba(0.1, 0.3, 0.9, 0.4).into(),
        ..default()
    },
    TabMenuWrapper(TabMenuType::DeathRecap),
)}

#[derive(Component, Debug)]
pub struct StoreMain;

pub fn store() -> impl Bundle {(
    NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Row,
            size: Size::new(Val::Percent(45.), Val::Percent(75.)),
            padding: UiRect{
                top: Val::Px(20.),
                ..default()
            },
            position: UiRect {
                left: Val::Px(210.0),
                top: Val::Percent(10.0),
                ..default()
            },
            ..default()
        },
        background_color: Color::rgba(0.2, 0.2, 0.4, 1.0).into(),
        visibility: Visibility::Hidden,
        ..default()
    },
    Draggable,
    StoreMain,
    Name::new("Store"),
)}

#[derive(Component, Debug)]
pub struct Draggable;

#[derive(Component, Debug)]
pub struct DragHandle;

pub fn drag_bar() -> impl Bundle{(
    NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            align_items: AlignItems::Baseline,
            size: Size::new(Val::Percent(100.), Val::Px(20.)),
            position: UiRect {
                left: Val::Px(0.0),
                right: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            ..default()
        },
        z_index: ZIndex::Global(6),
        background_color: Color::rgba(0.6, 0.7, 0.4, 0.2).into(),
        transform: Transform {
            translation: Vec3::new(0., 0., 1.),
            ..default()
        },
        focus_policy: FocusPolicy::Block,
        ..default()
    },
    DragHandle,
    Interaction::default(),
    Name::new("DragBar")
)}

pub fn list_categories() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(20.), Val::Percent(100.)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::rgba(1., 0.1, 0.2, 0.3).into(),
        ..default()
    }
)}

pub fn category() -> impl Bundle {(
    ButtonBundle {
        background_color: Color::rgb(0.15, 0.15, 0.15).into(),
        z_index: ZIndex::Global(2),
        ..default()
    }
)}

pub fn category_text(text: String, fonts: &Res<Fonts>) -> impl Bundle {(
    TextBundle::from_section(
        text.to_owned(),
        TextStyle {
            font: fonts.exo_regular.clone(),
            font_size: 15.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    )
)}

pub fn list_items() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(60.), Val::Percent(100.)),
            ..default()
        },
        background_color: Color::rgba(1., 0.5, 0.2, 0.3).into(),
        ..default()
    },
    Name::new("Store List"),
)}
pub fn inspector() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(20.), Val::Percent(100.)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        background_color: Color::rgba(1., 0.1, 0.6, 0.3).into(),
        ..default()
    }
)}

pub fn item_image(items: &Res<Items>, item: Item) -> impl Bundle {(
    ImageBundle {
        style: Style {
            size: Size::new(Val::Px(36.), Val::Px(36.)),
            ..default()
        },
        image: match item{
            Item::Arondight => items.arondight.clone().into(),
            Item::SoulReaver => items.soul_reaver.clone().into(),
            Item::HiddenDagger => items.hidden_dagger.clone().into(),
        },
        focus_policy: FocusPolicy::Block,
        ..default()
    },
    Draggable,
    DragHandle,
    Interaction::default(),
)}

#[derive(Component, Debug)]
pub struct HoverButton;

pub fn button() -> impl Bundle {(
    ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(150.0), Val::Px(65.0)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        background_color: Color::rgb(0.15, 0.15, 0.15).into(),
        z_index: ZIndex::Global(2),
        ..default()
    },
    HoverButton,    
)}

pub fn button_text(text: String, fonts: &Res<Fonts>) -> impl Bundle {(
    TextBundle::from_section(
        text.to_owned(),
        TextStyle {
            font: fonts.exo_regular.clone(),
            font_size: 40.0,
            color: Color::rgb(0.9, 0.9, 0.9),
        },
    )
)}

pub fn gold_text(fonts: &Res<Fonts>) -> impl Bundle {(
    (TextBundle {
        style: Style {
            margin : UiRect{
                top: Val::Px(10.),
                ..default()
            },
            ..default()
        },
        text: Text::from_section(
            "200",
            TextStyle {
                font: fonts.exo_light.clone(),
                font_size: 30.0,
                color: Color::YELLOW,
            },
        ),
        ..default()
    },
    GoldInhand
    )
)}

#[derive(Component, Debug)]
pub struct GoldInhand;


#[derive(Component)]
pub struct FollowIn3d(pub Entity);

pub fn follow_wrapper(entity: Entity) -> impl Bundle{(
        Name::new("Floating Text"),
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                size: Size::new(Val::Px(100.), Val::Px(30.)),
                justify_content: JustifyContent::Center,
                ..default()
            },
            z_index: ZIndex::Global(-1),
            ..default()
        },
        FollowIn3d(entity)
)}

pub fn follow_inner_text(damage: String, fonts: &Res<Fonts>) -> impl Bundle{(
        TextBundle {
            text: Text::from_section(
                damage.to_string(),
                TextStyle {
                    font: fonts.exo_regular.clone(),
                    font_size: 20.,
                    color: Color::YELLOW,
                },
            ),
            ..default()
        },
)}

pub fn template() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(300.), Val::Px(200.)),
            position_type: PositionType::Absolute,
            ..default()
        },
        background_color: Color::rgba(1., 0.1, 0.2, 0.4).into(),
        ..default()
    }
)}