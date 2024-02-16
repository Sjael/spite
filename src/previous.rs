use crate::prelude::*;

#[derive(Component, Deref)]
pub struct Previous<T>(pub T);

pub fn previous<T>(mut commands: Commands, mut query: Query<(Entity, Option<&mut Previous<T>>, &T)>)
where
    T: Component + Clone,
{
    for (entity, mut previous, current) in &mut query {
        match previous {
            Some(ref mut previous) => {
                previous.0 = current.clone();
            }
            None => {
                commands.entity(entity).insert(Previous::<T>(current.clone()));
            }
        }
    }
}
