use bevy::prelude::*;
use bevy_tweening::TweenCompleted;

use crate::{
    assets::Fonts,
    prelude::{ActorState, InGameSet},
    session::director::GameModeDetails,
    ui::{editing_ui_label, kill_notification, InGameClock, Killfeed, MinimapPlayerIcon, TweenEvents},
};

pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                minimap_track,
                tick_clock_ui,
                killfeed_update,
                kill_notif_cleanup,
            )
                .in_set(InGameSet::Update),
        );
    }
}

fn minimap_track(
    mut arrow_query: Query<&mut Style, With<MinimapPlayerIcon>>,
    trackables: Query<&GlobalTransform, With<Trackable>>,
) {
    let Ok(mut style) = arrow_query.get_single_mut() else { return };
    for tracking in trackables.iter() {
        let (player_left, player_top) = (tracking.translation().x, tracking.translation().z);
        style.left = Val::Px(player_left);
        style.top = Val::Px(player_top);
    }
}

fn killfeed_update(
    mut commands: Commands,
    changed_states: Query<(&ActorState, Option<&Name>), Changed<ActorState>>, // Add player id so we can increment kda for that player
    killfeed_query: Query<Entity, With<Killfeed>>,
    fonts: Res<Fonts>,
) {
    for (actor_state, opt_name) in changed_states.iter() {
        if actor_state.is_alive() {
            continue
        }
        let Ok(killfeed) = killfeed_query.get_single() else { return };
        let notification = commands.spawn(kill_notification()).id();
        if let Some(name) = opt_name {
            let name_ent = commands.spawn(editing_ui_label(name, &fonts)).id();
            commands.entity(notification).push_children(&[name_ent]);
        }
        commands.entity(killfeed).push_children(&[notification]);
    }
}

fn kill_notif_cleanup(mut commands: Commands, mut tween_events: EventReader<TweenCompleted>) {
    for ev in tween_events.read() {
        use TweenEvents::*;
        match TweenEvents::try_from(ev.user_data) {
            Ok(KillNotifEnded) => {
                commands.entity(ev.entity).despawn_recursive();
            }
            Err(_) | Ok(_) => (),
        }
    }
}

fn tick_clock_ui(time: Res<Time>, mut clock: Query<&mut Text, With<InGameClock>>, game_details: Res<GameModeDetails>) {
    // Shouldnt do abs calculations every tick probably just 1/s, increment the
    // seconds, increment minute if above 60
    let Ok(mut text) = clock.get_single_mut() else { return };
    let elapsed = time.elapsed().as_secs() as i32;
    let adjusted = game_details.start_timer + elapsed;
    let mut sign = "";
    let minute = (adjusted / 60).abs();
    let second = (adjusted % 60).abs();
    if adjusted < 0 {
        sign = "-";
    }

    text.sections[0].value = format!("{}{:02}:{:02}", sign, minute, second);
}

#[derive(Component)]
pub struct Trackable;
