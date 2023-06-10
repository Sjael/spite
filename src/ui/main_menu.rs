use bevy::{prelude::*, app::AppExit};
use crate::{ui::styles::*, assets::Fonts, GameState};






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
            MainMenuButton{
                button_type: MainMenuOption::Play
            },
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
            MainMenuButton{
                button_type: MainMenuOption::Settings
            },
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
            MainMenuButton{
                button_type: MainMenuOption::Exit
            },
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


pub fn main_menu_buttons(
    mut interaction_query: Query<
        (&MainMenuButton, &Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_writer: EventWriter<AppExit>,
) {
    for (button, interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Clicked => {
                match button.button_type {
                    MainMenuOption::Play => {
                        next_state.set(GameState::InGame);
                    },
                    MainMenuOption::Settings => {

                    }
                    MainMenuOption::Exit => {
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

pub fn cleanup(mut commands: Commands, root: Query<Entity, With<MainMenuRoot>>) {
    for entity in root.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Component)]
pub struct MainMenuRoot;

#[derive(PartialEq, Eq)]
pub enum MainMenuOption{
    Play,
    Settings,
    Exit,
}

#[derive(Component)]
pub struct MainMenuButton{
    pub button_type: MainMenuOption
}

pub fn exit_game_main_menu(
    kb: Res<Input<KeyCode>>,
    mut app_exit_writer: EventWriter<AppExit>,
){
    if kb.just_pressed(KeyCode::Escape){
        app_exit_writer.send(AppExit);
    }
}
