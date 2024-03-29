use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    actor::player::{LocalPlayerId, Player},
    assets::Fonts,
    inventory::Inventory,
    prelude::{InGameSet, Team, TEAM_1},
    session::team::TeamRoster,
    ui::ui_bundles::{plain_text, scoreboard_entry, KDAText, PersonalKDA, ScoreboardUI},
};

pub struct ScoreboardPlugin;
impl Plugin for ScoreboardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Scoreboard::default());

        app.add_systems(
            Update,
            (populate_scoreboard, update_kda).in_set(InGameSet::Update),
        );
    }
}

fn populate_scoreboard(
    teams: Query<(&Team, &TeamRoster)>,
    mut commands: Commands,
    scoreboard: Query<Entity, Added<ScoreboardUI>>,
    fonts: Res<Fonts>,
) {
    let Ok(scoreboard_ui) = scoreboard.get_single() else { return }; // else spawn scoreboard?
    commands.entity(scoreboard_ui).despawn_descendants();
    for (team, players) in teams.iter() {
        let mut color = Color::rgba(0.3, 0.15, 0.1, 0.95);
        if team == &TEAM_1 {
            color = Color::rgba(0.15, 0.15, 0.2, 0.95);
        }
        for player in players.iter() {
            commands.entity(scoreboard_ui).with_children(|parent| {
                parent.spawn(scoreboard_entry(color)).with_children(|parent| {
                    parent.spawn(plain_text(player.id.clone().to_string(), 14, &fonts));
                });
                parent.spawn(scoreboard_entry(color)).with_children(|parent| {
                    parent.spawn(plain_text("0 / 0 / 0", 14, &fonts)).insert(KDAText);
                });
            });
        }
    }
}

fn update_kda(
    mut kda_query: Query<&mut Text, With<PersonalKDA>>,
    //mut scoreboard_kda_query: Query<&mut Text, (With<KDAText>, Without<PersonalKDA>)>,
    scoreboard: Res<Scoreboard>,
    local_player_id: Res<LocalPlayerId>,
) {
    if scoreboard.is_changed() {
        let Ok(mut kda_text) = kda_query.get_single_mut() else { return };
        for (player, info) in scoreboard.0.iter() {
            if *player != **local_player_id {
                continue
            }
            kda_text.sections[0].value = format!(
                "{} / {} / {}",
                info.kda.kills, info.kda.deaths, info.kda.assists
            );
        }
    }
}

#[derive(Resource, Default)]
pub struct Scoreboard(pub HashMap<Player, PlayerInfo>);

#[derive(Default)]
pub struct PlayerInfo {
    pub kda: KDA,
    pub inv: Inventory,
    pub logs: LoggedNumbers,
    // account_name: String,
    // account_icon: Image,
    // ping: u32,
    // class: GameClass,
}

#[derive(Default)]
pub struct KDA {
    pub kills: u32,
    pub deaths: u32,
    pub assists: u32,
}

#[derive(Default)]
pub struct LoggedNumbers {
    pub gold_acquired: u32,
    pub damage_dealt: u32,
    pub damage_taken: u32,
    pub damage_mitigated: u32,
    pub healing_dealt: u32,
}
