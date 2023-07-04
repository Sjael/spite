use std::time::Duration;

use bevy::{prelude::*, ui::FocusPolicy};
use bevy_tweening::{Animator,  lens::{ UiPositionLens, UiBackgroundColorLens, TextColorLens}, EaseFunction, Tween, Delay};

use crate::{ability::{AbilityTooltip, Ability}, assets::{Icons, Items, Fonts, Images}, item::Item, crowd_control::CCType};

use super::styles::*;




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



#[derive(Component, Debug)]
pub struct EditableUI;

#[derive(Component, Debug)]
pub struct EditingUIHandle;
#[derive(Component, Debug)]
pub struct EditingUILabel;
#[derive(Component, Debug)]
pub struct UiForEditingUi;

pub fn editing_ui_handle() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            position_type: PositionType::Absolute,
            ..default()
        },
        background_color: Color::rgba(0.1, 0.2, 0.8, 0.5).into(),
        focus_policy: FocusPolicy::Block,
        ..default()
    },
    DragHandle,
    Interaction::default(),
    EditingUIHandle,
)}

pub fn editing_ui_label(text: impl Into<String>, fonts: &Res<Fonts>) -> impl Bundle{
    let text = text.into();
    (
    TextBundle {
        style: Style {
            margin: UiRect::all(Val::Px(15.)),
            ..default()
        },
        text: Text::from_section(
            text,
            TextStyle {
                font: fonts.exo_bold.clone(),
                font_size: 16.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    },
    EditingUILabel,
)}

pub fn editable_ui_wrapper() -> impl Bundle{(
    NodeBundle{
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            position_type: PositionType::Absolute,
            ..default()
        },
        ..default()
    },
    EditableUI,
    Name::new("Editable wrapper"),
)}

pub fn editing_ui() -> impl Bundle{(
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            position: UiRect{
                right: Val::Px(30.),
                bottom: Val::Px(30.),
                ..default()
            },
            ..default()
        },
        z_index: ZIndex::Global(11),
        ..default()
    },
    UiForEditingUi,
)}


#[derive(Component, Debug)]
pub struct HealthBarText;

#[derive(Component, Debug)]
pub struct HealthBarUI;

#[derive(Component, Debug)]
pub struct ResourceBarUI;

#[derive(Component, Debug)]
pub struct ResourceBarText;

#[derive(Component, Debug)]
pub struct CooldownIconText;

#[derive(Component, Debug)]
pub struct InGameClock;
#[derive(Component, Debug)]
pub struct RespawnHolder;
#[derive(Component, Debug)]
pub struct RespawnText;

pub fn respawn_holder() -> impl Bundle{(
    NodeBundle{
        style: Style {
            size: Size::new(Val::Px(200.), Val::Px(150.)),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position: UiRect {
                left: Val::Percent(20.),
                bottom: Val::Percent(10.),
                ..default()
            },
            ..default()
        },
        visibility: Visibility::Hidden,
        ..default()
    },
    RespawnHolder,
    Name::new("Respawn Text"),
)}

pub fn respawn_text(fonts: &Res<Fonts>) -> impl Bundle{(
    TextBundle {
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
                value: "0".to_string(),
                style: TextStyle {
                    font: fonts.exo_bold.clone(),
                    font_size: 36.0,
                    color: Color::WHITE,
                },
            },
        ]).with_alignment(TextAlignment::Center),
        ..default()
    },
    RespawnText,
)}

pub fn minimap_holder() -> impl Bundle{(
    NodeBundle{
        style: Style {
            size: Size::new(Val::Px(300.), Val::Px(300.)),
            position_type: PositionType::Absolute,
            margin: UiRect {
                right: Val::Percent(20.),
                ..UiRect::all(Val::Auto)
            },
            position: UiRect {
                right: Val::Percent(20.),
                ..UiRect::all(Val::Px(0.))
            },
            ..default()
        },
        ..default()
    },
    Name::new("Minimap"),
)}

#[derive(Component)]
pub struct Minimap;

pub fn minimap(images: &Res<Images>) -> impl Bundle {(
    ImageBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            ..default()
        },
        image: images.minimap.clone().into(),
        ..default()
    },
    Interaction::None,
    Minimap,
    Name::new("Minimap Image"),
)}

#[derive(Component)]
pub struct MinimapPlayerIcon;
pub fn minimap_arrow(images: &Res<Images>) -> impl Bundle{(    
    ImageBundle {
        style: Style {
            max_size: Size::new(Val::Px(16.), Val::Px(16.)),
            position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Auto),
            position: UiRect::all(Val::Px(0.)),
            ..default()
        },
        image: images.circle.clone().into(),
        ..default()
    },
    MinimapPlayerIcon,
    Name::new("Arrow"),
)}

pub fn killfeed_holder() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(150.), Val::Percent(45.)),
            position_type: PositionType::Absolute,
            margin: UiRect {
                right: Val::Px(30.),
                ..UiRect::all(Val::Auto)
            },
            ..default()
        },
        ..default()
    },
    Name::new("Killfeed"),
)}

#[derive(Component)]
pub struct Killfeed;

pub fn killfeed() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    },
    Killfeed,
    Name::new("Killfeed list"),
)}

#[derive(Component)]
pub struct KillNotification;
pub fn kill_notification() -> impl Bundle{
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
    FloatingDamageEnded = 1,
}

impl TryFrom<u64> for TweenEvents{
    type Error = String;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        use TweenEvents::*;
        match value{
            0 => Ok(KillNotifEnded),
            1 => Ok(FloatingDamageEnded),
            _ => Err("invalid TweenEvents index".to_string()),
        }
    }
}

#[derive(Component)]
pub struct FollowIn3d{
    pub leader: Entity,
    pub last_seen: Option<Transform>,
}

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
    FollowIn3d{
        leader: entity,
        last_seen: None,
    }
)}

pub fn follow_inner_text(damage: String, fonts: &Res<Fonts>) -> impl Bundle{
    let killfeed_offset = 40.;
    let delay_seconds = 1;
    let text_color = Color::WHITE;
    let tween_pos = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(500),
        UiPositionLens {
            start: UiRect{
                top:Val::Px(killfeed_offset),
                ..default()
            },
            end: UiRect{
                top:Val::Px(0.),
                ..default()
            },
        },
    );
    let tween_opac_in = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(500),
        TextColorLens {
            start: *text_color.clone().set_a(0.0),
            end: *text_color.clone().set_a(1.0),
            section: 0,
        },
    );
    let tween_opac_out = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(250),
        TextColorLens {
            start: *text_color.clone().set_a(1.0),
            end: *text_color.clone().set_a(0.0),
            section: 0,
        },
    ).with_completed_event(TweenEvents::FloatingDamageEnded as u64);
    (
    TextBundle {
        text: Text::from_section(
            damage.to_string(),
            TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: 20.,
                color: text_color,
            },
        ),
        ..default()
    },
    Animator::new(tween_pos),
    Animator::new(tween_opac_in
        .then(Delay::new(
            Duration::from_secs(delay_seconds),
        ))
        .then(tween_opac_out)
    ),
)}


pub fn floating_health_bar() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(120.), Val::Px(30.)),
            position_type: PositionType::Absolute,
            padding: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        ..default()
    },
    Name::new("Health bar"),
)}

pub fn bottom_left_ui_holder() -> impl Bundle {(
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
    Name::new("Stats / Build / KDA"),
)}

pub fn bottom_left_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            ..default()
        },
        ..default()
    },
    Name::new("Bottom Left Ui"),
)}
pub fn stats_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(60.), Val::Percent(100.)),
            flex_direction:FlexDirection::Column,
            ..default()
        },
        //background_color: Color::rgba(0.9, 0.1, 0.9, 0.2).into(),
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
        //background_color: Color::rgba(0.9, 0.6, 0.1, 0.2).into(),
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
        //background_color: Color::rgba(0.1, 0.6, 0.9, 0.2).into(),
        ..default()
    },
    Name::new("KDA"),
)}

#[derive(Component)]
pub struct TeammateThumbs;

pub fn team_thumbs_holder() -> impl Bundle {(
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
        ..default()
    },
    Name::new("Team thumbs"),
)}

pub fn team_thumbs() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.), Val::Percent(100.)),
            ..default()
        },
        //background_color: Color::rgba(0.9, 0.9, 0.2, 0.4).into(),
        ..default()
    },
    TeammateThumbs,
    Name::new("Team thumbs UI"),
)}


pub fn header_holder() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(35.), Val::Px(80.)),
            position_type: PositionType::Absolute,
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            ..default()
        },
        ..default()
    },
    Name::new("Header"),
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
        //background_color: Color::rgba(0.1, 0.2, 0.7, 0.4).into(),
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

pub fn character_ui() -> impl Bundle {(
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
    PlayerUI,
    Name::new("Character UI"),
)}

#[derive(Component)]
pub struct PlayerUI;
pub fn player_bottom_container() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(320.), Val::Auto),
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
        //background_color: Color::rgba(0.1, 0.7, 0.7, 0.4).into(),
        ..default()
    },
)}

pub fn effect_bar() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Auto),
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

#[derive(Component)]
pub struct BuffBar;
pub fn buff_bar() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(50.), Val::Percent(100.)),
            flex_wrap: FlexWrap::WrapReverse, 
            ..default()
        },
        //background_color: Color::rgba(0.1, 0.1, 1., 0.4).into(),
        ..default()
    },
    BuffBar,
)}

#[derive(Component)]
pub struct DebuffBar;
pub fn debuff_bar() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(50.), Val::Percent(100.)),
            flex_wrap: FlexWrap::WrapReverse, 
            ..default()
        },
        //background_color: Color::rgba(1., 0.1, 0.2, 0.4).into(),
        ..default()
    },
    DebuffBar,
)}

#[derive(Component)]
pub struct BuffId{
    pub id: String,
}

pub fn buff_holder(time: f32, id: String) -> impl Bundle{(
    NodeBundle {
        style: Style {
            margin: UiRect {
                right: Val::Px(7.),
                top: Val::Px(10.),
                ..default()
            },
            flex_direction: FlexDirection::Column,
            ..default()
        },
        ..default()
    },
    BuffId { id },
    DespawnTimer(
        Timer::new(Duration::from_millis((time * 1000.0)as u64), TimerMode::Once)
    ),
)}

pub fn buff_border(is_buff: bool) -> impl Bundle{
    let color: Color;
    if is_buff{
        color = ALLY_COLOR;
    }else {
        color = ENEMY_COLOR;
    }
    (
    NodeBundle {
        style: Style {
            size: Size::new(Val::Px(28.), Val::Px(28.)),
            ..default()
        },
        background_color: color.into(),
        ..default()
    },
)}

pub fn buff_image(ability: Ability, icons: &Res<Icons>,) -> impl Bundle{(
    ImageBundle {
        style: Style {
            size: Size::new(Val::Percent(90.), Val::Percent(90.)),
            margin: UiRect::all(Val::Auto),
            ..default()
        },
        image: match ability{
            Ability::Frostbolt => icons.frostbolt.clone().into(),
            Ability::Fireball => icons.fireball.clone().into(),
            Ability::Dash => icons.dash.clone().into(),
            _ => icons.basic_attack.clone().into(),
        },
        background_color: Color::rgba(1.0, 1.0, 1.0, 0.95).into(),
        ..default()
    },
    Interaction::None,
    Name::new("Buff Image"),
)}

#[derive(Component)]
pub struct BuffDurationText;


pub fn buff_timer(fonts: &Res<Fonts>, is_buff: bool) -> impl Bundle{
    let color: Color;
    if is_buff{
        color = ALLY_COLOR;
    }else {
        color = ENEMY_COLOR;
    }
    (
    TextBundle {
        style: Style {
            margin: UiRect{
                bottom: Val::Px(5.0),
                ..UiRect::all(Val::Auto)
            },
            ..default()
        },
        text: Text::from_section(
            "8",
            TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: 16.0,
                color,
            },
        ),
        ..default()
    },
    BuffDurationText,
    Name::new("Buff Duration"),
)}

#[derive(Component)]
pub struct BuffStackNumber;

pub fn buff_stacks(fonts: &Res<Fonts>) -> impl Bundle{(
    TextBundle {
        style: Style {
            position_type: PositionType::Absolute,
            position: UiRect{
                bottom: Val::Px(5.0),
                right: Val::Px(5.0),
                ..default()
            },
            ..default()
        },
        text: Text::from_section(
            "2",
            TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: 14.0,
                color: Color::WHITE,
            },
        ),
        visibility: Visibility::Hidden,
        ..default()
    },
    BuffStackNumber,
    Name::new("Buff stack number"),
)}

pub fn player_bars_wrapper() -> impl Bundle{(
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
            gap: Size::height(Val::Px(4.0)),
            ..default()
        },
        ..default()
    }
)}

pub fn bar_wrapper(height: f32) -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(100.0), Val::Px(height)),
            ..default()
        },
        background_color: Color::rgba(0.05, 0.05, 0.1, 0.9).into(),
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
    HealthBarUI,
)}
pub fn resource_bar_color() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(60.0), Val::Percent(100.0)),
            ..default()
        },
        background_color: Color::rgb(0.92, 0.24, 0.01).into(),
        ..default()
    },
    ResourceBarUI,
)}

pub fn bar_text_wrapper() -> impl Bundle{(    
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
                font: fonts.exo_semibold.clone(),
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
            gap:Size::new(Val::Px(6.), Val::Auto),
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
            size: Size::new(Val::Px(40.), Val::Px(40.)),
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
                font_size: 24.0,
                color: Color::WHITE,
            },
        ),
        ..default()
    },
    CooldownIconText,
    Name::new("Cooldown Text"),
)}

pub fn cast_bar_holder() -> impl Bundle {(
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            position_type: PositionType::Absolute,
            size: Size::new(Val::Px(200.0), Val::Px(44.0)),
            margin: UiRect::all(Val::Auto),
            position: UiRect{
                bottom: Val::Px(-30.0),
                ..default()
            },
            ..default()
        },
        visibility: Visibility::Hidden,
        ..default()
    },
    CastBar,
    Name::new("Castbar Holder"),
)}
#[derive(Component)]
pub struct CastBar;

pub fn cast_bar() -> impl Bundle {(
    NodeBundle {
        style: Style { 
            size: Size::new(Val::Percent(100.0), Val::Px(2.0)),
            ..default()
        },
        background_color: Color::rgba(0.05, 0.05, 0.1, 0.5).into(),
        ..default()
    },
)}
#[derive(Component)]
pub struct CastBarFill;
pub fn cast_bar_fill() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(60.0), Val::Percent(100.0)),
            ..default()
        },
        background_color: Color::rgba(1.0, 1.0, 0.3, 0.9).into(),
        ..default()
    },
    CastBarFill,
)}

pub fn cc_holder() -> impl Bundle {(
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            position_type: PositionType::Absolute,
            size: Size::new(Val::Px(100.0), Val::Px(40.0)),
            margin: UiRect::all(Val::Auto),
            position: UiRect{
                bottom: Val::Px(100.0),
                ..default()
            },
            ..default()
        },
        ..default()
    },
    CCSelf,
    Name::new("CC Holder"),
)}
pub fn cc_holder_top() -> impl Bundle {(
    NodeBundle {
        style: Style {
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            gap: Size::all(Val::Px(6.0)),
            ..default()
        },
        ..default()
    },
)}
#[derive(Component)]
pub struct CCSelf;
#[derive(Component)]
pub struct CCSelfLabel;

#[derive(Component)]
pub struct CCIconSelf;
pub fn cc_icon(cctype: CCType, icons: &Res<Icons>) -> impl Bundle{(
    ImageBundle {
        style: Style {
            size: Size::new(Val::Px(22.), Val::Px(22.)),
            ..default()
        },
        image: cctype.get_icon(icons).into(),
        ..default()
    },
    Name::new("CC Icon"),
)}

pub fn cc_bar() -> impl Bundle {(
    NodeBundle {
        style: Style { 
            size: Size::new(Val::Percent(100.0), Val::Px(6.0)),
            ..default()
        },
        background_color: Color::rgba(0.05, 0.05, 0.1, 0.9).into(),
        ..default()
    },
)}
#[derive(Component)]
pub struct CCBarSelfFill;
pub fn cc_bar_fill() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(60.0), Val::Percent(100.0)),
            ..default()
        },
        background_color: Color::rgba(1.0, 1.0, 1.0, 0.9).into(),
        ..default()
    },
    CCBarSelfFill,
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
    info: &AbilityTooltip,
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
            "driving strike" => icons.dash.clone().into(),
            "fireball" => icons.fireball.clone().into(),
            _ => icons.basic_attack.clone().into(),
        },
        ..default()
    },
    TooltipContents::Image,
)}



#[derive(Component, Debug)]
pub struct InGameMenuUi;
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
    InGameMenuUi,
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

#[derive(Component, Default, Eq, PartialEq)]
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
    TabMenuType::DamageLog,
    DamageLog,
)}

#[derive(Component)]
pub struct OutgoingLogUi;
#[derive(Component)]
pub struct IncomingLogUi;

pub fn log_outgoing() -> impl Bundle {(    
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::FlexEnd,
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
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::FlexEnd,
            flex_grow: 1.0,
            ..default()
        },
        background_color: Color::rgba(0.3, 0.14, 0.1, 0.99).into(),
        ..default()
    },
    IncomingLogUi,
)}

pub fn damage_entry(damage: String, fonts: &Res<Fonts>,) -> impl Bundle {
    let text = damage.to_string();
    (
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
    TabMenuType::Scoreboard,
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
    TabMenuType::Abilities,
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
    TabMenuType::DeathRecap,
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
    Draggable::BoundByParent(100),
    StoreMain,
    Name::new("Store"),
)}

#[derive(Component, Debug, PartialEq)]
pub enum Draggable{
    Unbound,
    BoundByParent(i32), // so you can go outside if you want
}

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

pub fn category_text(text: impl Into<String>, fonts: &Res<Fonts>) -> impl Bundle {
    let text = text.into();
    (
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
    Draggable::BoundByParent(0),
    DragHandle,
    Interaction::default(),
)}

#[derive(Component, Debug)]
pub struct HoverButton;

pub fn button() -> impl Bundle {(
    ButtonBundle {
        style: Style {
            size: Size::new(Val::Px(120.0), Val::Px(50.0)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            margin: UiRect{
                bottom: Val::Px(10.),
                ..default()
            },
            ..default()
        },
        background_color: Color::rgb(0.15, 0.15, 0.15).into(),
        z_index: ZIndex::Global(2),
        ..default()
    },
    HoverButton,    
)}

pub fn button_text(text: impl Into<String>, fonts: &Res<Fonts>) -> impl Bundle {
    let text = text.into();
    (
    TextBundle::from_section(
        text.to_owned(),
        TextStyle {
            font: fonts.exo_regular.clone(),
            font_size: 36.0,
            color: Color::rgb(0.9, 0.9, 1.0),
        },
    )
)}

pub fn gold_text(fonts: &Res<Fonts>) -> impl Bundle {(
    TextBundle {
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
    GoldInhand,
)}

pub fn plain_text(text: impl Into<String>, size: u32, fonts: &Res<Fonts>) -> impl Bundle {
    let text = text.into();
    (
    TextBundle::from_section(
        text.to_owned(),
        TextStyle {
            font: fonts.exo_semibold.clone(),
            font_size: size as f32,
            color: Color::rgb(1.0, 1.0, 1.0),
        },
    )
)}

#[derive(Component, Debug)]
pub struct GoldInhand;

#[derive(Component)]
pub struct HealthBarHolder;

#[derive(Component)]
pub struct ObjectiveName;
#[derive(Component)]
pub struct ObjectiveHealth;
pub fn objective_health_bar_holder() -> impl Bundle {(
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::SpaceBetween,
            position_type: PositionType::Absolute,
            size: Size::new(Val::Px(150.0), Val::Px(50.0)),
            margin: UiRect::all(Val::Auto),
            position: UiRect{
                bottom: Val::Percent(30.0),
                ..default()
            },
            ..default()
        },
        visibility: Visibility::Hidden, 
        ..default()
    },
    ObjectiveHealth,
    Name::new("Objective Health Holder"),
)}

#[derive(Component)]
pub struct ObjectiveHealthFill;
pub fn objective_health_fill() -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(60.0), Val::Percent(100.0)),
            ..default()
        },
        background_color: Color::rgba(1.0, 0.2, 0.2, 0.9).into(),
        ..default()
    },
    ObjectiveHealthFill,
)}

#[derive(Component)]
pub struct HealthBar;

pub fn bar_fill(color: Color) -> impl Bundle {(
    NodeBundle {
        style: Style {
            size: Size::new(Val::Percent(60.0), Val::Percent(100.0)),
            ..default()
        },
        background_color: color.into(),
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