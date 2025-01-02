use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::dynamics::Velocity;

use crate::{
    climbing::Climber, colliders::ColliderBundle, ground_detection::GroundDetection,
    inventory::Inventory,
};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_sheet("player.png", 16, 16, 7, 1, 0, 0, 0)]
    pub sprite: Sprite,
    #[from_entity_instance]
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    #[worldly]
    pub worldly: Worldly,
    pub climber: Climber,
    pub ground_detection: GroundDetection,

    // Build Items Component manually by using `impl From<&EntityInstance>`
    #[from_entity_instance]
    items: Inventory,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

// Player jump 2 block vertically, and jump 4 horizontally but just barely.
pub fn player_movement(
    input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Velocity, &mut Climber, &GroundDetection), With<Player>>,
) {
    for (mut velocity, mut climber, ground_detection) in &mut query {
        let right = if input.pressed(KeyCode::ArrowRight) {
            1.
        } else {
            0.
        };
        let left = if input.pressed(KeyCode::ArrowLeft) {
            1.
        } else {
            0.
        };

        velocity.linvel.x = (right - left) * 150.;

        if climber.intersecting_climbables.is_empty() {
            climber.climbing = false;
        } else if input.just_pressed(KeyCode::ArrowUp) || input.just_pressed(KeyCode::ArrowDown) {
            climber.climbing = true;
        }

        if climber.climbing {
            let up = if input.pressed(KeyCode::ArrowUp) {
                1.
            } else {
                0.
            };
            let down = if input.pressed(KeyCode::ArrowDown) {
                1.
            } else {
                0.
            };

            velocity.linvel.y = (up - down) * 200.;
        }

        if input.just_pressed(KeyCode::Space) {
            if ground_detection.on_ground || climber.climbing {
                // If the playing is moving horizontally, then make their jump
                // slighlty less powerful
                if velocity.linvel.x == 0. {
                    velocity.linvel.y = 400.;
                } else {
                    velocity.linvel.y = 390.;
                }
                climber.climbing = false;
            }
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, player_movement)
            .register_ldtk_entity::<PlayerBundle>("Player");
    }
}
