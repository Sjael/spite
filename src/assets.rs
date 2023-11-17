//! Preload assets via collections

use bevy::utils::HashMap;

use bevy_asset_loader::prelude::*;
use crate::prelude::*;
use crate::GameState;

#[derive(AssetCollection, Resource)]
pub struct Icons {
    #[asset(path = "icons/frostbolt.png")]
    pub frostbolt: Handle<Image>,
    #[asset(path = "icons/basicattack.png")]
    pub basic_attack: Handle<Image>,
    #[asset(path = "icons/dash.png")]
    pub dash: Handle<Image>,
    #[asset(path = "icons/fireball.png")]
    pub fireball: Handle<Image>,
    #[asset(path = "icons/swarm.png")]
    pub swarm: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct Items {
    #[asset(path = "icons/items/Arondight_T3.png")]
    pub arondight: Handle<Image>,
    #[asset(path = "icons/items/HiddenDagger_T1.png")]
    pub hidden_dagger: Handle<Image>,
    #[asset(path = "icons/items/SoulReaver_T3.png")]
    pub soul_reaver: Handle<Image>,
    #[asset(path = "icons/items/Spellbook_T1.png")]
    pub book_of_souls: Handle<Image>,
    #[asset(path = "icons/items/Witchblade_T3.png")]
    pub witchblade: Handle<Image>,
    #[asset(path = "icons/items/DruidStone_T1.png")]
    pub druid_stone: Handle<Image>,
    #[asset(path = "icons/items/Polynomicon_T3.png")]
    pub polynomicon: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct Fonts {
    #[asset(path = "fonts/Exo/Exo2-Black.otf")]
    pub exo_black: Handle<Font>,
    #[asset(path = "fonts/Exo/Exo2-Bold.otf")]
    pub exo_bold: Handle<Font>,
    #[asset(path = "fonts/Exo/Exo2-SemiBold.otf")]
    pub exo_semibold: Handle<Font>,
    #[asset(path = "fonts/Exo/Exo2-Medium.otf")]
    pub exo_medium: Handle<Font>,
    #[asset(path = "fonts/Exo/Exo2-Regular.otf")]
    pub exo_regular: Handle<Font>,
    #[asset(path = "fonts/Exo/Exo2-Light.otf")]
    pub exo_light: Handle<Font>,
    #[asset(path = "fonts/Exo/Exo2-Thin.otf")]
    pub exo_thin: Handle<Font>,
}

#[derive(AssetCollection, Resource)]
pub struct Images {
    #[asset(path = "images/minimap.png")]
    pub minimap: Handle<Image>,
    #[asset(path = "images/circle.png")]
    pub circle: Handle<Image>,
    #[asset(path = "images/enemy_tower.png")]
    pub enemy_tower: Handle<Image>,
    #[asset(path = "images/friendly_tower.png")]
    pub friendly_tower: Handle<Image>,
    #[asset(path = "images/default.png")]
    pub default: Handle<Image>,
}

#[derive(AssetCollection, Resource)]
pub struct Models {
    #[asset(path = "models/skybox.gltf#Scene0")]
    pub skybox: Handle<Scene>,
}
#[derive(AssetCollection, Resource)]
pub struct Scenes {
    #[asset(path = "scenes/arenaMap2.glb#Scene0")]
    pub arena_map: Handle<Scene>,
}

#[derive(Resource)]
pub struct MaterialPresets(pub HashMap<String, Handle<StandardMaterial>>);

pub struct GameAssetPlugin;
impl Plugin for GameAssetPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(GameState::Loading).continue_to_state(GameState::MainMenu),
        )
        .add_collection_to_loading_state::<_, Icons>(GameState::Loading)
        .add_collection_to_loading_state::<_, Fonts>(GameState::Loading)
        .add_collection_to_loading_state::<_, Images>(GameState::Loading)
        .add_collection_to_loading_state::<_, Scenes>(GameState::Loading)
        .add_collection_to_loading_state::<_, Models>(GameState::Loading)
        .add_collection_to_loading_state::<_, Items>(GameState::Loading);

        app.add_systems(Startup, load_presets);
    }
}

fn load_presets(mut commands: Commands, mut materials: ResMut<Assets<StandardMaterial>>) {
    let colors = vec![
        ("white", Color::WHITE),
        ("blue", Color::BLUE),
        ("red", Color::RED),
    ];
    let mut presets = HashMap::new();
    for (color_string, color) in colors {
        presets.insert(color_string.into(), materials.add(color.into()));
    }
    commands.insert_resource(MaterialPresets(presets));
}
