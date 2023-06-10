use bevy::prelude::*;


#[derive(Debug, Clone, Reflect, FromReflect)]
pub struct CCInfo{
    pub cctype: CCType,
    pub duration: f32,
}

#[derive(Debug, Clone, Eq, PartialEq, Reflect, FromReflect, Hash)]
pub enum CCType{
    Stun,
    Root,
    Fear,
    Disarm,
    Silence,
    //Slow, change to buff since affects a stat, proper CC's are for absolutes
    Cripple,
}

impl std::fmt::Display for CCType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}