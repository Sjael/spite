

use derive_more::Display;

use bevy::prelude::*;

#[derive(Component, Reflect, Clone, Debug, Default, Display, Eq, PartialEq, Hash)]
#[reflect(Component)]
pub enum Item {
    Arondight,
    #[default]
    SoulReaver,
    HiddenDagger,
}