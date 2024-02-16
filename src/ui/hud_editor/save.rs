#![allow(unused)]
use bevy::{
    ecs::{reflect::ReflectResource, system::Resource},
    reflect::Reflect,
    utils::HashMap,
};
use ron::ser::{to_string_pretty, PrettyConfig};
use serde::{Deserialize, Serialize};

pub const LAYOUT_PATH: &'static str = "hud_layout.ron";

#[derive(Clone, Deserialize, Debug, Default, Reflect, Resource, Serialize)]
#[reflect(Resource)]
pub struct Layout(pub HashMap<String, Offset>);

impl Layout {
    pub fn read_file() -> Layout {
        let file = match std::fs::File::open(LAYOUT_PATH) {
            Ok(file) => file,
            Err(err) => return Layout::default(),
        };
        ron::de::from_reader(file).unwrap_or_default()
    }

    pub fn save(self) {
        if self.0.is_empty() {
            if std::path::Path::new(LAYOUT_PATH).exists() {
                std::fs::remove_file(LAYOUT_PATH).expect("file shouldve existed");
            }
            return
        }
        use std::io::Write;
        let mut file = std::fs::File::create(LAYOUT_PATH).expect("create hud_layout file");
        let pretty = PrettyConfig::new()
            .depth_limit(2)
            .separate_tuple_members(true)
            .enumerate_arrays(true);
        let str = to_string_pretty(&self, pretty).expect("Serialization failed");

        file.write_all(str.as_bytes()).expect("write to hud_layout.ron");
        file.flush().expect("could not flush to hud_layout.ron");
    }
}

#[derive(Clone, Deserialize, Debug, Reflect, Serialize)]
pub struct Offset {
    pub left: f32,
    pub top: f32,
    pub scale: f32,
}

impl Default for Offset {
    fn default() -> Self {
        Offset {
            left: 0.0,
            top: 0.0,
            scale: 1.0,
        }
    }
}

impl PartialEq<Offset> for Offset {
    fn eq(&self, other: &Self) -> bool {
        self.left as u32 == other.left as u32
            && self.top as u32 == other.top as u32
            && self.scale as u32 == self.scale as u32
    }
}

const TEST_LAYOUT: &str = r#"
({
    "stats": Offset (
        left: 55.0,
        top: 10.0,
        scale: 1.0,
    ),
    "killfeed": Offset (
        left: 25.0,
        top: 3.0,
        scale: 1.0,
    ),
    "map": Offset (
        left: 0.0,
        top: 0.0,
        scale: 1.5,
    ),
})"#;

pub fn read_test_layout() -> Layout {
    ron::from_str(TEST_LAYOUT).unwrap_or_default()
}
