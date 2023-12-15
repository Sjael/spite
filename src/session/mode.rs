use bevy::prelude::*;

#[derive(Default, Resource)]
pub enum GameMode {
    #[default]
    Arena,
    Tutorial,
    Conquest,
    Practice,
}
