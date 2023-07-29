use std::time::Duration;

use bevy::{prelude::*, ui::FocusPolicy};
use bevy_tweening::{Animator,  lens::{ UiPositionLens, UiBackgroundColorLens, TextColorLens}, EaseFunction, Tween, Delay};

use crate::{ability::{AbilityTooltip, Ability}, assets::{Icons, Items, Fonts, Images}, item::Item, actor::crowd_control::CCType};

use rand::Rng;
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
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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
            right: Val::Px(30.),
            bottom: Val::Px(30.),
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
            width: Val::Px(200.),
            height: Val::Px(150.),
            position_type: PositionType::Absolute,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            left: Val::Percent(20.),
            bottom: Val::Percent(10.),
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
            width: Val::Px(300.),
            height: Val::Px(300.),
            position_type: PositionType::Absolute,
            margin: UiRect {
                right: Val::Percent(20.),
                ..UiRect::all(Val::Auto)
            },
            right: Val::Percent(20.),
            top: Val::Px(0.),
            left: Val::Px(0.),
            bottom: Val::Px(0.),
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
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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
            max_width: Val::Px(16.),
            max_height: Val::Px(16.),
            position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Auto),
            top: Val::Px(0.),
            bottom: Val::Px(0.),
            left: Val::Px(0.),
            right: Val::Px(0.),
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
            width: Val::Px(150.),
            height: Val::Percent(45.),
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
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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
            width: Val::Percent(100.),
            height: Val::Px(64.),
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
            width: Val::Px(100.),
            height: Val::Px(30.),
            margin: UiRect{
                left:Val::Px(-50.0),
                ..default()
            },
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

pub fn follow_inner_text(damage: String, fonts: &Res<Fonts>, color: Color) -> impl Bundle{
    let mut rng = rand::thread_rng();
    let top_offset = 40.;
    let start_horizontal = rng.gen_range(-30..30);
    let spread = 30;
    let end_horizontal = rng.gen_range(start_horizontal - spread..start_horizontal + spread);
    let delay_seconds = 1;
    let text_color = color.clone();
    let tween_pos = Tween::new(
        EaseFunction::QuadraticIn,
        Duration::from_millis(500),
        UiPositionLens {
            start: UiRect{
                top:Val::Px(top_offset),
                left:Val::Px(start_horizontal as f32),
                ..default()
            },
            end: UiRect{
                top:Val::Px(0.),
                left:Val::Px(end_horizontal as f32),
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
                font_size: 25.,
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
            width: Val::Px(120.),
            height: Val::Px(30.),
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
            width: Val::Px(220.),
            height: Val::Px(140.),
            position_type: PositionType::Absolute,
            bottom: Val::Px(40.),
            left: Val::Px(40.),
            ..default()
        },
        ..default()
    },
    Name::new("Stats / Build / KDA"),
)}

pub fn bottom_left_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        ..default()
    },
    Name::new("Bottom Left Ui"),
)}
pub fn stats_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Px(60.),
            height: Val::Percent(100.),
            flex_direction:FlexDirection::Column,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    },
    Name::new("Stats"),
)}
pub fn build_and_kda() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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
            width: Val::Percent(100.),
            height: Val::Percent(80.),
            display: Display::Grid,
            grid_template_columns: RepeatedGridTrack::auto(3),
            grid_template_rows: RepeatedGridTrack::auto(2),
            column_gap: Val::Px(8.),
            row_gap: Val::Px(8.),
            padding: UiRect::all(Val::Px(12.0)),
            ..default()
        },
        ..default()
    },
    BuildUI,
    Name::new("BuildUI"),
)}

pub fn build_slot() -> impl Bundle {(
    NodeBundle {
        style: Style {
            height: Val::Px(48.),
            aspect_ratio: Some(1.0),
            padding: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: Color::rgba(0., 0., 0., 0.1).into(),
        ..default()
    },
    Name::new("Build slot"),
    DropSlot,
)}
#[derive(Component)]
pub struct DropSlot;

pub fn kda_ui() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Px(20.),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            ..default()
        },
        ..default()
    },
    Name::new("KDA"),
)}

#[derive(Component)]
pub struct KDAText;
#[derive(Component)]
pub struct PersonalKDA;

#[derive(Component)]
pub struct TeammateThumbs;

pub fn team_thumbs_holder() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Px(200.),
            height: Val::Px(80.),
            position_type: PositionType::Absolute,
            top: Val::Px(40.),
            left: Val::Px(40.),
            ..default()
        },
        ..default()
    },
    Name::new("Team thumbs"),
)}

pub fn team_thumbs() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            ..default()
        },
        ..default()
    },
    TeammateThumbs,
    Name::new("Team thumbs UI"),
)}


pub fn header_holder() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(35.),
            height: Val::Px(80.),
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
            width: Val::Percent(35.),
            height: Val::Px(80.),
            position_type: PositionType::Absolute,
            justify_content:JustifyContent::Center,
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            ..default()
        },
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
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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
            width: Val::Px(320.),
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.),
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            flex_direction: FlexDirection::Column,
            align_items:AlignItems::Center,
            ..default()
        },
        ..default()
    },
)}

pub fn effect_bar() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Auto,
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
            width: Val::Percent(50.),
            height: Val::Percent(100.),
            flex_wrap: FlexWrap::WrapReverse, 
            ..default()
        },
        ..default()
    },
    BuffBar,
)}

#[derive(Component)]
pub struct DebuffBar;
pub fn debuff_bar() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(50.),
            height: Val::Percent(100.),
            flex_wrap: FlexWrap::WrapReverse, 
            ..default()
        },
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
            width: Val::Px(28.),
            height: Val::Px(28.),
            ..default()
        },
        background_color: color.into(),
        ..default()
    },
)}

pub fn buff_image(ability: Ability, icons: &Res<Icons>,) -> impl Bundle{(
    ImageBundle {
        style: Style {
            width: Val::Percent(90.),
            height: Val::Percent(90.),
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
            bottom: Val::Px(5.0),
            right: Val::Px(5.0),
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
            width: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            margin: UiRect {
                bottom: Val::Px(10.),
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            row_gap: Val::Px(4.0),
            ..default()
        },
        ..default()
    }
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
            column_gap: Val::Px(6.),
            ..default()
        },
        ..default()
    },
    AbilityHolder,
    Name::new("Ability Holder"),
)}

pub fn ability_image(handle: Handle<Image>) -> impl Bundle {(
    ImageBundle {
        style: Style {
            width: Val::Px(40.),
            height: Val::Px(40.),
            ..default()
        },
        image: handle.into(),
        ..default()
    },
    Interaction::None,
    Name::new("Ability Image"),
)}


pub fn custom_image(handle: Handle<Image>, size: u32) -> impl Bundle {(
    ImageBundle {
        style: Style {
            width: Val::Px(size as f32),
            height: Val::Px(size as f32),
            ..default()
        },
        image: handle.into(),
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
            top:Val::Px(-2.0),
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
            width: Val::Px(200.),
            height: Val::Px(44.),
            margin: UiRect::all(Val::Auto),
            bottom: Val::Px(-30.0),
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

#[derive(Component)]
pub struct CastBarFill;

pub fn cc_holder() -> impl Bundle {(
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            position_type: PositionType::Absolute,
            width: Val::Px(100.),
            height: Val::Px(40.),
            margin: UiRect::all(Val::Auto),
            bottom: Val::Px(100.0),
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
            column_gap: Val::Px(6.),
            row_gap: Val::Px(6.),
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
            width: Val::Px(22.),
            height: Val::Px(22.),
            ..default()
        },
        image: cctype.get_icon(icons).into(),
        ..default()
    },
    Name::new("CC Icon"),
)}


#[derive(Component)]
pub struct CCBarSelfFill;

pub fn tooltip() -> impl Bundle{(
    NodeBundle{
        style:Style {
            position_type: PositionType::Absolute,
            min_width: Val::Px(150.),
            min_height: Val::Px(100.), // only cus adding to an empty makes it weird 1 frame
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
            max_width: Val::Px(320.),
            max_height: Val::Auto,
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
            width: Val::Px(64.),
            height: Val::Px(64.),
            position_type: PositionType::Absolute,
            left: Val::Px(-74.),
            top: Val::Px(0.),
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
            width: Val::Percent(20.),
            height: Val::Percent(50.),
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
            width: Val::Percent(70.),
            height: Val::Percent(70.),
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
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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
            width: Val::Percent(50.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::FlexEnd,
            padding: UiRect::all(Val::Px(20.0)),
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
            width: Val::Percent(50.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::ColumnReverse,
            justify_content: JustifyContent::FlexEnd,
            padding: UiRect::all(Val::Px(20.0)),
            column_gap: Val::Px(20.0),
            ..default()
        },
        background_color: Color::rgba(0.3, 0.14, 0.1, 0.99).into(),
        ..default()
    },
    IncomingLogUi,
)}

pub fn despawn_wrapper(delay: u32) -> impl Bundle {(
    NodeBundle {
        ..default()
    },
    Name::new("Despawen"),
    DespawnTimer(
        Timer::new(Duration::from_secs(delay.into()), TimerMode::Once)
    )
)}

pub fn column_wrapper() -> impl Bundle {(
    NodeBundle {
        style: Style {
            flex_direction: FlexDirection::Column,
            column_gap: Val::Px(10.0),
            ..default()
        },
        ..default()
    },
    Name::new("Damage column"),
)}

pub fn damage_entry() -> impl Bundle {(
    NodeBundle {
        style: Style {
            margin : UiRect{
                top: Val::Px(12.),
                ..default()
            },
            align_content: AlignContent::Center,
            align_self:AlignSelf::Start,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },
        ..default()
    },
    Name::new("Damage Entry"),
)}

pub fn thin_image(image: Handle<Image>) -> impl Bundle{(
    ImageBundle {
        style: Style {
            width: Val::Px(55.),
            height: Val::Px(22.),
            align_self:AlignSelf::Center,
            ..default()
        },
        image: image.into(),
        ..default()
    },
    Name::new("CC Icon"),
)}

#[derive(Component, Debug)]
pub struct DespawnTimer(pub Timer);

#[derive(Component, Debug)]
pub struct ScoreboardUI;
pub fn scoreboard() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            position_type: PositionType::Absolute,
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
            /*
            grid_auto_flow: GridAutoFlow::ColumnDense,
            grid_template_rows: vec![
                GridTrack::fr(1.0),
            ],
            grid_auto_rows: vec![GridTrack::flex(1.0), GridTrack::flex(1.0)],
             */
            padding: UiRect::all(Val::Px(15.)),
            ..default()
        },
        background_color: Color::rgba(0.14, 0.14, 0.2, 0.99).into(),
        ..default()
    },
    TabMenuType::Scoreboard,
    ScoreboardUI,
)}

pub fn scoreboard_entry(color: Color) -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            justify_content: JustifyContent::Center,
            align_content: AlignContent::Center,
            ..default()
        },
        background_color: color.into(),
        ..default()
    },
)}


pub fn abilities_panel() -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Px(300.),
            height: Val::Px(200.),
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
            width: Val::Px(300.),
            height: Val::Px(200.),
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
            width: Val::Percent(45.),
            height: Val::Percent(75.),
            padding: UiRect{
                top: Val::Px(20.),
                ..default()
            },
            left: Val::Px(210.0),
            top: Val::Percent(10.0),
            ..default()
        },
        background_color: Color::rgba(0.2, 0.2, 0.4, 1.0).into(),
        visibility: Visibility::Hidden,
        ..default()
    },
    Draggable::BoundByParent(20),
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
            width: Val::Percent(100.),
            height: Val::Px(20.),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
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
            width: Val::Percent(20.),
            height: Val::Percent(100.),
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
            width: Val::Percent(60.),
            height: Val::Percent(100.),
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
            width: Val::Percent(20.),
            height: Val::Percent(100.),
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
            width: Val::Px(36.),
            height: Val::Px(36.),
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

pub fn item_image_build(items: &Res<Items>, item: Item) -> impl Bundle {(
    ImageBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
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
    Draggable::Unbound,
    DragHandle,
    Interaction::default(),
)}

#[derive(Component, Debug)]
pub struct HoverButton;

pub fn button() -> impl Bundle {(
    ButtonBundle {
        style: Style {
            width: Val::Px(120.),
            height: Val::Px(50.),
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
    TextBundle {
        style: Style {
            align_self:AlignSelf::Center,
            ..default()
        },
        text: Text::from_section(
            text,
            TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: size as f32,
                color: Color::WHITE.into(),
            },
        ),
        ..default()
    },
)}

pub fn color_text(text: impl Into<String>, size: u32, fonts: &Res<Fonts>, color: Color) -> impl Bundle {
    let text = text.into();
    (       
    TextBundle {
        style: Style {
            align_self:AlignSelf::Center,
            ..default()
        },
        text: Text::from_section(
            text,
            TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: size as f32,
                color: color.into(),
            },
        ),
        ..default()
    },
)}

#[derive(Component, Debug)]
pub struct GoldInhand;

#[derive(Component)]
pub struct HealthBarHolder(pub Entity);

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
            width: Val::Px(150.),
            height: Val::Px(50.),
            margin: UiRect::all(Val::Auto),
            bottom: Val::Percent(30.0),
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

#[derive(Component)]
pub struct HealthBar;

pub fn bar_fill(color: Color) -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(60.),
            height: Val::Percent(100.),
            ..default()
        },
        background_color: color.into(),
        ..default()
    },
)}


pub fn bar_background(height: f32) -> impl Bundle {(
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Px(height),
            ..default()
        },
        background_color: Color::rgba(0.05, 0.05, 0.1, 0.9).into(),
        ..default()
    }
)}


pub fn bar_text_wrapper() -> impl Bundle{(    
    NodeBundle {
        style: Style {
            width: Val::Percent(100.),
            height: Val::Percent(100.),
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            align_items: AlignItems::Center,
            ..default()
        },
        ..default()
    },
)}

pub fn custom_text(fonts: &Res<Fonts>, size: f32, offset: f32) -> impl Bundle {(
    TextBundle {
        style: Style {
            margin:UiRect::top(Val::Px(offset)),
            ..default()
        },
        text: Text::from_section(
            "200",
            TextStyle {
                font: fonts.exo_semibold.clone(),
                font_size: size,
                color: Color::WHITE,
            },
        ),
        ..default()
    },
)}

pub fn template() -> impl Bundle {(
    NodeBundle {
        style: Style {
            height: Val::Px(200.),
            width:Val::Px(300.),
            position_type: PositionType::Absolute,
            ..default()
        },
        background_color: Color::rgba(1., 0.1, 0.2, 0.4).into(),
        ..default()
    }
)}