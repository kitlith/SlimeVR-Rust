use std::collections::HashMap;

use crate::model::bone::Bone;
use crate::model::BoneKind;
use crate::model::BoneMap;
use crate::RGBA;

use eyre::Context;
use eyre::Result;
use lazy_static::lazy_static;
use ovr_overlay::overlay::OverlayManager;
use stackvec::TryCollect;

use super::bone::Isometry;

pub type BoneArena = BoneMap<Bone>;

lazy_static! {
    static ref DEFAULT_COLORS: BoneMap<RGBA> = {
        use BoneKind::*;
        HashMap::from([
            (Head, RGBA::SILVER),
            (Neck, RGBA::GRAY),
            (Chest, RGBA::OLIVE),
            (Waist, RGBA::LIME),
            (Hip, RGBA::GREEN),
            (ThighL, RGBA::AQUA),
            (ThighR, RGBA::AQUA),
            (AnkleL, RGBA::TEAL),
            (AnkleR, RGBA::TEAL),
            (FootL, RGBA::BLUE),
            (FootR, RGBA::BLUE),
            (UpperArmL, RGBA::RED),
            (UpperArmR, RGBA::RED),
            (ForearmL, RGBA::PURPLE),
            (ForearmR, RGBA::PURPLE),
            (WristL, RGBA::FUCHSIA),
            (WristR, RGBA::FUCHSIA),
        ])
        .try_into()
        .unwrap()
    };
}

const BONE_RADIUS: f32 = 0.01;

/// Builder for the [`Skeleton`].
pub struct SkeletonBuilder {
    colors: Option<BoneMap<Option<RGBA>>>,
    key: String,
    bone_radius: f32,
    bone_lengths: Option<BoneMap<f32>>,
}
impl SkeletonBuilder {
    #[allow(dead_code)]
    pub fn build(self, overlay_manager: &mut OverlayManager) -> Result<Skeleton> {
        let colors = if let Some(colors) = self.colors {
            colors
        } else {
            Default::default()
        };
        let colors: BoneMap<RGBA> = colors
            .into_iter()
            .map(|(kind, maybe_color)| (kind, maybe_color.unwrap_or_else(|| DEFAULT_COLORS[kind])))
            .try_collect()
            .unwrap();

        let bone_lengths = self
            .bone_lengths
            .unwrap_or_else(|| BoneMap::new([0.1; BoneKind::NUM_TYPES]));

        let mut bones = Vec::new();
        for (kind, color) in colors {
            let bone = Bone::new(
                overlay_manager,
                color,
                Default::default(),
                format!("{}: {kind:?}", self.key),
                self.bone_radius,
                bone_lengths[kind],
            )?;
            bones.push((kind, bone));
        }
        let bones: BoneArena = bones.into_iter().try_collect().unwrap();
        Ok(Skeleton::new(bones))
    }
}
impl Default for SkeletonBuilder {
    fn default() -> Self {
        Self {
            colors: None,
            key: String::from("slimevr"),
            bone_radius: BONE_RADIUS,
            bone_lengths: None,
        }
    }
}

pub struct Skeleton {
    bones: BoneArena,
    is_visible: bool,
}
#[allow(dead_code)]
impl Skeleton {
    pub fn new(bones: BoneArena) -> Self {
        let mut result = Self {
            bones,
            is_visible: false,
        };
        // We explicitly set all bones to invisible, to reduce code brittleness.
        result.set_visibility(false);
        result
    }

    pub fn set_isometry(&mut self, bone: BoneKind, iso: Isometry) {
        let bone = &mut self.bones[bone];
        bone.set_isometry(iso);
    }

    pub fn set_length(&mut self, bone: BoneKind, len: f32) {
        let bone = &mut self.bones[bone];
        bone.set_length(len);
    }

    pub fn update_render(&mut self, bone: BoneKind, mngr: &mut OverlayManager) -> eyre::Result<()> {
        let bone = &mut self.bones[bone];
        bone.update_render(mngr)
            .wrap_err("could not update render for bone")
    }

    pub fn visibility(&self) -> bool {
        self.is_visible
    }

    pub fn set_visibility(&mut self, is_visible: bool) {
        if self.is_visible == is_visible {
            return;
        }
        self.is_visible = is_visible;
        for (_, bone) in self.bones.iter_mut() {
            bone.set_visibility(is_visible);
        }
    }
}