use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::colliders::ColliderBundle;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Spike;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct SpikeBundle {
    pub spike: Spike,
    #[sprite_sheet]
    pub sprite_sheet: Sprite,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
}

pub struct SpikePlugin;

impl Plugin for SpikePlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<SpikeBundle>("Spike");
    }
}
