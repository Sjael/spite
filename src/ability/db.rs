use std::collections::HashMap;

use bevy::{ecs::system::Res, ui::UiImage, utils::default};

use crate::{
    ability::{
        shape::AbilityShape,
        timeline::{AreaTimeline, CastStage},
        Ability, DamageType, TagInfo,
    },
    buff::BuffInfo,
    crowd_control::{CCInfo, CCKind},
    prelude::Icons,
    stats::Stat,
};

// All these methods can be easily replaced with calls to a DB table w/ ability info eventually
// DB much easier for hotfixes
impl Ability {
    pub fn get_cost(&self) -> i32 {
        match self {
            Ability::Frostbolt => 0,
            Ability::Fireball => 1,
            Ability::Dash => 2,
            Ability::BasicAttack => 0,
            _ => 1,
        }
    }

    pub fn get_cooldown(&self) -> f32 {
        match self {
            Ability::Dash => 7.,
            Ability::Frostbolt => 3.5,
            Ability::Fireball => 4.,
            Ability::BasicAttack => 0.8,
            _ => 3.,
        }
    }

    pub fn on_reticle(&self) -> bool {
        match self {
            Ability::Fireball => true,
            Ability::Bomb => true,
            _ => false,
        }
    }

    pub fn get_image(&self, icons: &Res<Icons>) -> UiImage {
        let image = match self {
            Ability::Frostbolt => &icons.frostbolt,
            Ability::Fireball => &icons.fireball,
            Ability::Dash => &icons.dash,
            Ability::Bomb => &icons.swarm,
            _ => &icons.basic_attack,
        };
        image.clone().into()
    }

    pub fn get_name(&self) -> String {
        let str = match self {
            Ability::Frostbolt => "Frostbolt",
            Ability::Fireball => "Fireball",
            Ability::Bomb => "Rain of Fire",
            Ability::Dash => "Driving Strike",
            _ => "Ability",
        };
        str.to_string()
    }

    pub fn get_description(&self) -> String {
        let str = match self {
            Ability::Frostbolt => "Cold as fuck",
            Ability::Dash => {
                "Hercules delivers a mighty strike, driving all enemies back, 
                damaging and Stunning them. Hercules is immune to Knockback during the dash."
            }
            Ability::Bomb => "Gamer move",
            _ => "A very boring attack",
        };
        str.to_string()
    }

    pub fn get_damage_type(&self) -> DamageType {
        match self {
            Ability::Frostbolt => DamageType::Magical,
            Ability::Fireball => DamageType::Magical,
            Ability::Bomb => DamageType::Physical,
            _ => DamageType::True,
        }
    }

    pub fn get_scaling(&self) -> u32 {
        match self {
            Ability::Frostbolt => 30,
            _ => 40,
        }
    }

    pub fn get_timeline_blueprint(&self) -> HashMap<CastStage, f32> {
        let map = match self {
            Ability::Bomb => vec![
                (CastStage::Input, 0.1),
                (CastStage::Casted, 0.1),
                (CastStage::Windup, 0.2),
                (CastStage::Firing, 2.0),
                (CastStage::Spindown, 0.2),
            ],
            _ => vec![
                (CastStage::Input, 0.05),
                (CastStage::Casted, 0.1),
                (CastStage::Windup, 0.1),
                (CastStage::Firing, 1.0),
                (CastStage::Spindown, 0.1),
            ],
        };
        map.into_iter().collect()
    }

    pub fn get_area_timeline(&self) -> AreaTimeline {
        let map = self.get_timeline_blueprint();
        AreaTimeline::new_at_stage(map, CastStage::Input)
    }

    pub fn get_deployed_lifetime(&self) -> f32 {
        self.get_timeline_blueprint()
            .iter()
            .filter(|(stage, _)| **stage != CastStage::Input && **stage != CastStage::Casted)
            .fold(0.0, |x, i| x + i.1)
    }

    pub fn get_speed(&self) -> f32 {
        match self {
            Ability::Frostbolt => 18.0,
            Ability::Fireball => 22.0,
            Ability::Bomb => 6.0,
            Ability::BasicAttack => 30.0,
            Ability::Dash => 10.0,
            _ => 0.0,
        }
    }
    pub fn get_length(&self) -> f32 {
        match self {
            Ability::Frostbolt => 0.8,
            Ability::Fireball => 2.0,
            Ability::Bomb => 2.0,
            Ability::BasicAttack => 2.0,
            Ability::Dash => 10.0,
            _ => 0.0,
        }
    }

    pub fn get_shape(&self) -> AbilityShape {
        match self {
            Ability::Frostbolt => AbilityShape::Rectangle {
                length: 0.8,
                width: 0.5,
            },
            Ability::Fireball => AbilityShape::Arc {
                radius: 1.,
                angle: 360.,
            },
            Ability::Bomb => AbilityShape::Arc {
                radius: 1.5,
                angle: 360.,
            },
            Ability::BasicAttack => AbilityShape::default(),
            Ability::Dash => todo!(),
            _ => AbilityShape::default(),
        }
    }

    pub fn has_movement(&self) -> bool {
        match self {
            Ability::Bomb => false,
            _ => true,
        }
    }
    pub fn has_filter(&self) -> bool {
        match self {
            Ability::Bomb => true,
            _ => false,
        }
    }

    pub fn get_tags(&self) -> Vec<TagInfo> {
        match self {
            Ability::Frostbolt => vec![
                TagInfo::Damage(38.0),
                TagInfo::CC(CCInfo {
                    cckind: CCKind::Stun,
                    duration: 1.0,
                }),
                TagInfo::Buff(BuffInfo {
                    stat: Stat::Health.into(),
                    amount: 10.0,
                    duration: 10.0,
                    ..default()
                }),
            ],
            Ability::Fireball => vec![TagInfo::Damage(11.0)],
            Ability::Bomb => vec![TagInfo::Damage(16.0)],
            Ability::BasicAttack => vec![TagInfo::Damage(11.0)],
            Ability::Dash => vec![
                TagInfo::Damage(25.0),
                TagInfo::CC(CCInfo {
                    cckind: CCKind::Root,
                    duration: 1.0,
                }),
            ],
            _ => vec![TagInfo::Damage(100.0)],
        }
    }
}
