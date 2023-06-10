use bevy::{prelude::*, app::AppExit};
use crate::{ui::styles::*, assets::Fonts, GameState};

use super::ButtonAction;






pub fn spawn_main_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fonts: Res<Fonts>,
){
    commands.spawn((
        NodeBundle{
            style: Style{
                size: Size::new(
                    Val::Percent(100.0),
                    Val::Percent(100.0),
                ),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                gap: Size::new(
                    Val::Percent(10.0),
                    Val::Px(20.),
                ),
                ..default()
            },
            ..default()
        },
        MainMenuRoot,
    )).with_children(|parent|{
        parent.spawn((
            ButtonBundle{
                style:Style{
                    size:Size::new(
                        Val::Percent(15.),
                        Val::Percent(20.),
                    ),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            ButtonAction::Play,
        )).with_children(|parent|{
            parent.spawn((
                TextBundle::from_section(
                    "Play", 
                    TextStyle{
                        font: fonts.exo_bold.clone(),
                        font_size: 40.0,
                        color: Color::BLACK,
                    }
                ),
            ));
        });

        parent.spawn((
            ButtonBundle{
                style:Style{
                    size:Size::new(
                        Val::Percent(15.),
                        Val::Percent(20.),
                    ),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            ButtonAction::Settings,
        )).with_children(|parent|{
            parent.spawn((
                TextBundle::from_section(
                    "Settings", 
                    TextStyle{
                        font: fonts.exo_bold.clone(),
                        font_size: 40.0,
                        color: Color::BLACK,
                    }
                ),
            ));
        });

        parent.spawn((
            ButtonBundle{
                style:Style{
                    size:Size::new(
                        Val::Percent(15.),
                        Val::Percent(20.),
                    ),
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            ButtonAction::Exit,
        )).with_children(|parent|{
            parent.spawn((
                TextBundle::from_section(
                    "Exit", 
                    TextStyle{
                        font: fonts.exo_bold.clone(),
                        font_size: 40.0,
                        color: Color::BLACK,
                    }
                ),
            ));
        });

    });
}


pub fn cleanup(mut commands: Commands, root: Query<Entity, With<MainMenuRoot>>) {
    for entity in root.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct MainMenuRoot;



pub fn exit_game_main_menu(
    kb: Res<Input<KeyCode>>,
    mut app_exit_writer: EventWriter<AppExit>,
){
    if kb.just_pressed(KeyCode::Escape){
        app_exit_writer.send(AppExit);
    }
}
