use crate::stats::{Stat};
use bevy::prelude::*;


#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub struct BuffInfoTest{
    pub stat:Stat,
    pub duration: f32,
}

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub struct BuffInfo{
    pub stat:Stat,
    pub max_stacks: u32,
    pub duration: u32,
    pub falloff: StackFalloff,
    pub refresh: StackRefresh,
}

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub enum StackFalloff{
    None, // buff stacks drop one at a time,
    #[default]
    All, // buff stacks drop at the same time,
    Multiple(u32) // varying amount of falloff, pretty niche
}

#[derive(Default, Clone, Debug, Reflect, FromReflect)]
pub enum StackRefresh{
    None, // adding a stack doesnt refesh any,
    #[default]
    All, // adding a stack refreshes all
    Multiple(u32),
}

//
// PLACE WITH STAT MODULE
//