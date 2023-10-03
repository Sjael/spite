
use bevy::prelude::*;


#[derive(Reflect, Debug, Clone)]
pub struct Rank{
    pub current: u8,
    pub max: u8
}

impl Default for Rank{
    fn default() -> Self {
        Rank{
            current: 0,
            max: 5,
        }
    }
}