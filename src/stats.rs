
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::{marker::PhantomData, ops::MulAssign};
//use fixed::types::I40F24;
use std::ops::{Add, AddAssign, Deref, DerefMut, Mul};
use crate::on_gametick;


#[derive(Bundle)]
pub struct StatsBundle{
    health: Attribute::<Health>,
    resource: Attribute::<CharacterResource>,
}

impl Default for StatsBundle{
    fn default() -> Self{
        Self{
            health: Attribute::<Health>::new(250.),
            resource: Attribute::<CharacterResource>::new(6.),
        }
    }
}

// Use enum as stat instead of unit structs?
//
//
#[derive(Reflect, Debug, Default, Clone, FromReflect)]
pub enum Stat{
    #[default]
    Health,
    CharacterResource,
    Gold,
    Experience,
    PhysicalPower,
}
impl std::fmt::Display for Stat {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
//
// Makes buffing easier, dont need generic type in buffinfo
//

#[derive(Reflect, Debug)]
pub struct Health;
#[derive(Reflect, Debug)]
pub struct CharacterResource;
#[derive(Reflect, Debug)]
pub struct Gold;
#[derive(Reflect, Debug)]
pub struct Experience;
#[derive(Reflect, Debug)]
pub struct MovementSpeed;

pub struct StatsPlugin;
impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Attribute<Health>>();
        app.register_type::<Attribute<MovementSpeed>>();
        app.register_type::<Attribute<CharacterResource>>();
        app.add_systems((
            regen_unless_zero::<Health>.run_if(on_gametick),
            clamp_min::<Health>,
            clamp_max::<Health>,
            basic_modifiers::<Health>,
        ));
        app.add_systems((
            regen::<CharacterResource>.run_if(on_gametick),
            clamp_min::<CharacterResource>,
            clamp_max::<CharacterResource>,
            basic_modifiers::<CharacterResource>,
        ));
        app.add_systems((
            clamp_min::<MovementSpeed>,
            clamp_max::<MovementSpeed>,
            basic_modifiers::<MovementSpeed>,
        ));
    }
}

pub fn regen_unless_zero<A>(
    mut query: Query<(
        &mut Attribute<A>,
        &Attribute<Regen<A>>,
        Option<&Attribute<Max<A>>>,
    )>,
    time: Res<Time>,
) where
    A: 'static + Send + Sync + Reflect,
{
    for (mut attribute, regen, max) in query.iter_mut() {
        if *attribute.amount() <= 0.0 {
            continue;
        }
        let mut result = attribute.amount() + (regen.amount() * time.delta_seconds()) ;
        if let Some(max) = max {
            if result > *max.amount() {
                result = *max.amount();
            }
        }
        if *attribute.amount() != result {
            attribute.set_amount(result);
        }
    }
}

pub fn regen<A>(
    mut query: Query<(
        &mut Attribute<A>,
        &Attribute<Regen<A>>,
        Option<&Attribute<Max<A>>>,
    )>,
    time: Res<Time>,
) where
    A: 'static + Send + Sync + Reflect,
{
    for (mut attribute, regen, max) in query.iter_mut() {
        let mut result = attribute.amount() + (regen.amount() * time.delta_seconds()) ;
        if let Some(max) = max {
            if result > *max.amount() {
                result = *max.amount();
            }
        }
        if *attribute.amount() != result {
            attribute.set_amount(result);
        }
    }
}

pub fn clamp_max<A>(
    mut attributes: Query<
        (&mut Attribute<A>, &Attribute<Max<A>>),
        Or<(Changed<Attribute<A>>, Changed<Attribute<Max<A>>>)>,
    >,
) where
    A: 'static + Send + Sync + Reflect,
{
    for (mut current, max) in attributes.iter_mut() {
        if current.amount() > max.amount() {
            current.set_amount(*max.amount());
        }
    }
}

pub fn clamp_min<A>(
    mut attributes: Query<
        (&mut Attribute<A>, &Attribute<Min<A>>),
        Or<(Changed<Attribute<A>>, Changed<Attribute<Min<A>>>)>,
    >,
) where
    A: 'static + Send + Sync + Reflect,
{
    for (mut current, min) in attributes.iter_mut() {
        if current.amount() < min.amount() {
            current.set_amount(*min.amount());
        }
    }
}

pub fn basic_modifiers<A>(
    mut attributes: Query<
        (
            &mut Attribute<A>,
            Option<&Attribute<Base<A>>>,
            Option<&Attribute<Plus<A>>>,
            Option<&Attribute<Mult<A>>>,
        ),
        Or<(
            Changed<Attribute<Base<A>>>,
            Changed<Attribute<Plus<A>>>,
            Changed<Attribute<Mult<A>>>,
        )>,
    >,
) where
    A: 'static + Send + Sync + Reflect,
{
    for (mut current, base, plus, mult) in attributes.iter_mut() {
        let base = base.map(|base| base.amount()).unwrap_or(&0.0f32);
        let mult = mult.map(|mult| mult.amount()).unwrap_or(&1.0f32);
        let plus = plus.map(|plus| plus.amount()).unwrap_or(&0.0f32);
        let result = (base + plus) * mult;

        if *current.amount() != result {
            current.set_amount(result);
        }
    }
}


// Switch to I40f24 later for accuracy
pub type Amount = f32;

#[derive(Component, Debug, Copy, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
{
    amount: Amount,
    #[serde(skip, default)]
    #[reflect(ignore)]
    phantom: PhantomData<A>,
}

impl<A> Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
{
    pub fn new<I: Into<Amount>>(amount: I) -> Self {
        Self {
            amount: amount.into(),
            phantom: PhantomData,
        }
    }
    pub fn amount(&self) -> &Amount {
        &self.amount
    }

    pub fn set_amount(&mut self, amount: Amount) {
        self.amount = amount;
    }    
}


impl<A> From<f32> for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
{
    fn from(num: f32) -> Self{
        Self{
            amount: num.into(),
            phantom: PhantomData,
        }
    }
}

impl<A> Into<Amount> for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
{
    fn into(self) -> Amount {
        self.amount
    }
}

impl<Rhs, A> Mul<Rhs> for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
    Rhs: Into<Amount>,
{
    type Output = Self;
    fn mul(self, rhs: Rhs) -> Self::Output {
        let fixed = rhs.into();
        Attribute {
            amount: self.amount * fixed,
            phantom: PhantomData,
        }
    }
}

impl<Rhs, A> Add<Rhs> for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
    Rhs: Into<Amount>,
{
    type Output = Self;
    fn add(self, rhs: Rhs) -> Self::Output {
        let fixed = rhs.into();
        Attribute {
            amount: self.amount + fixed,
            phantom: PhantomData,
        }
    }
}

impl<Rhs, A> MulAssign<Rhs> for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
    Rhs: Into<Amount>,
{
    fn mul_assign(&mut self, rhs: Rhs) {
        self.amount = self.amount * rhs.into();
    }
}

impl<Rhs, A> AddAssign<Rhs> for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
    Rhs: Into<Amount>,
{
    fn add_assign(&mut self, rhs: Rhs) {
        self.amount = self.amount + rhs.into();
    }
}

impl<A> Clone for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
{
    fn clone(&self) -> Self {
        Self {
            amount: self.amount.clone(),
            phantom: PhantomData,
        }
    }
}

impl<A> Default for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
{
    fn default() -> Self {
        Self { amount: 0., phantom: PhantomData::<A> }
    }
}

impl<A> PartialEq for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
{
    fn eq(&self, other: &Self) -> bool {
        self.amount == other.amount
    }
}

impl<A> Deref for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
{
    type Target = Amount;
    fn deref(&self) -> &Self::Target {
        &self.amount
    }
}

impl<A> DerefMut for Attribute<A>
where
    A: 'static + Send + Sync + Reflect,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.amount
    }
}



#[derive(Default, Debug, Clone, Reflect)]
pub struct Min<A: 'static + Send + Sync + Reflect>(#[reflect(ignore)]PhantomData<A>);

#[derive(Default, Debug, Clone, Reflect)]
pub struct Max<A: 'static + Send + Sync + Reflect>(#[reflect(ignore)]PhantomData<A>);

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Reflect)]
pub struct Regen<A: 'static + Send + Sync + Reflect>(#[reflect(ignore)]PhantomData<A>);

#[derive(Default, Debug, Clone, Reflect)]
pub struct Base<A: 'static + Send + Sync + Reflect>(#[reflect(ignore)]PhantomData<A>);

#[derive(Default, Debug, Clone, Reflect)]
pub struct Plus<A: 'static + Send + Sync + Reflect>(#[reflect(ignore)]PhantomData<A>);

#[derive(Default, Debug, Clone, Reflect)]
pub struct Mult<A: 'static + Send + Sync + Reflect>(#[reflect(ignore)]PhantomData<A>);

