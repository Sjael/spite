use crate::prelude::*;

pub mod director;
pub mod mode;
pub mod team;

pub struct SessionPlugin;
impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(team::TeamPlugin)
            .add_plugins(director::DirectorPlugin);
    }
}
