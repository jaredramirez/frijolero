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

pub const ROTATION_CONSTRAINTS: LockedAxes = LockedAxes::ROTATION_LOCKED;

impl From<&EntityInstance> for ColliderBundle {
    fn from(entity_instance: &EntityInstance) -> ColliderBundle {
        match entity_instance.identifier.as_ref() {
            "Obstacle" => ColliderBundle {
                collider: Collider::cuboid(6., 6.),
                rigid_body: RigidBody::KinematicVelocityBased,
                rotation_constraints: ROTATION_CONSTRAINTS,
                ..Default::default()
            },
            "Mob" => ColliderBundle {
                collider: Collider::cuboid(6., 6.),
                rigid_body: RigidBody::KinematicVelocityBased,
                rotation_constraints: ROTATION_CONSTRAINTS,
                ..Default::default()
            },
            "Platform" => ColliderBundle {
                collider: Collider::cuboid(8., 8.),
                rigid_body: RigidBody::KinematicVelocityBased,
                friction: Friction::new(1.0),
                rotation_constraints: ROTATION_CONSTRAINTS,
                ..Default::default()
            },
            "Player_Respawn" => ColliderBundle {
                collider: Collider::cuboid(8., 8.),
                rigid_body: RigidBody::KinematicVelocityBased,
                friction: Friction::new(1.0),
                rotation_constraints: ROTATION_CONSTRAINTS,
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
        match int_grid_cell.value {
            // ladder
            4 => SensorBundle {
                collider: Collider::cuboid(8., 8.),
                sensor: Sensor,
                rotation_constraints: ROTATION_CONSTRAINTS,
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
                rotation_constraints: ROTATION_CONSTRAINTS,
                active_events: ActiveEvents::COLLISION_EVENTS,
            },
            _ => SensorBundle::default(),
        }
    }
}
