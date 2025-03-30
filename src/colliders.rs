use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity_scale: GravityScale,
    pub friction: Friction,
    pub density: ColliderMassProperties,
}

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                collider: Collider::compound(vec![
                    (Vect::new(0., 2.), 0., Collider::cuboid(6., 2.)),
                    (Vect::new(0., -4.), 0., Collider::cuboid(6., 4.)),
                ]),
                rigid_body: RigidBody::Dynamic,
                friction: Friction {
                    coefficient: 1.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                rotation_constraints,
                ..Default::default()
            },
            "Obstacle" => ColliderBundle {
                collider: Collider::cuboid(6., 6.),
                rigid_body: RigidBody::KinematicVelocityBased,
                rotation_constraints,
                ..Default::default()
            },
            "Mob" => ColliderBundle {
                collider: Collider::cuboid(6., 6.),
                rigid_body: RigidBody::KinematicVelocityBased,
                rotation_constraints,
                ..Default::default()
            },
            "Platform" => ColliderBundle {
                collider: Collider::cuboid(8., 8.),
                rigid_body: RigidBody::KinematicVelocityBased,
                friction: Friction::new(1.0),
                rotation_constraints,
                ..Default::default()
            },
            "Player_Respawn" => ColliderBundle {
                collider: Collider::cuboid(8., 8.),
                rigid_body: RigidBody::KinematicVelocityBased,
                friction: Friction::new(1.0),
                rotation_constraints,
                ..Default::default()
            },
            entity_id => {
                warn!(
                    "Unexpected entity when creating ColliderBundle {}",
                    entity_id
                );
                ColliderBundle::default()
            }
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct SensorBundle {
    pub collider: Collider,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub rotation_constraints: LockedAxes,
}

impl From<IntGridCell> for SensorBundle {
    fn from(int_grid_cell: IntGridCell) -> SensorBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;
        match int_grid_cell.value {
            // ladder
            4 => SensorBundle {
                collider: Collider::cuboid(8., 8.),
                sensor: Sensor,
                rotation_constraints,
                active_events: ActiveEvents::COLLISION_EVENTS,
            },
            // spike
            5 => SensorBundle {
                collider: Collider::compound(vec![(
                    Vect::new(0., -3.),
                    0.,
                    Collider::cuboid(8., 5.),
                )]),
                sensor: Sensor,
                rotation_constraints,
                active_events: ActiveEvents::COLLISION_EVENTS,
            },
            _ => SensorBundle::default(),
        }
    }
}
