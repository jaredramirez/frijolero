use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::CollisionEvent;

use crate::{colliders::SensorBundle, game_flow::RespawnLevelEvent};

/// a spike tag
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Spike;

/// a spike & it's sensor
#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct SpikeBundle {
    pub spike: Spike,
    #[from_int_grid_cell]
    pub sensor_bundle: SensorBundle,
}

/// put this on entites that you want to watch for spike collisions
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct SpikeDetection;

/// check if there is a collision between a SpikeDetection and a Spike
/// if so, respawn the level
pub fn detect_spike(
    to_detect: Query<Entity, With<SpikeDetection>>,
    spikes: Query<Entity, With<Spike>>,
    mut collisions: EventReader<CollisionEvent>,
    mut level_respawn_event: EventWriter<RespawnLevelEvent>,
) {
    for collision in collisions.read() {
        match collision {
            CollisionEvent::Started(collider_a, collider_b, _) => {
                if let (Ok(_), Ok(_)) = (to_detect.get(*collider_a), spikes.get(*collider_b)) {
                    level_respawn_event.send(RespawnLevelEvent::RespawnLevelEvent);
                }
                if let (Ok(_), Ok(_)) = (to_detect.get(*collider_b), spikes.get(*collider_a)) {
                    level_respawn_event.send(RespawnLevelEvent::RespawnLevelEvent);
                };
            }
            CollisionEvent::Stopped(_, _, _) => {}
        }
    }
}

pub struct SpikePlugin;

impl Plugin for SpikePlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_int_cell::<SpikeBundle>(5)
            .add_systems(Update, detect_spike);
    }
}
