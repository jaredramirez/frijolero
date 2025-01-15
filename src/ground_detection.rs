use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::HashMap;

use crate::platform::Platform;

#[derive(Component)]
pub struct GroundSensor {
    pub ground_detection_entity: Entity,
    pub intersecting_ground_entities: HashMap<Entity, GroundAttrs>,
}
#[derive(Eq, Hash, PartialEq, Clone, Default)]
pub struct GroundAttrs {
    pub is_platform: bool,
}

#[derive(Clone, Component)]
pub struct GroundDetection {
    pub grounded: Grounded,
    pub was_on_ground: bool,
}
impl GroundDetection {
    pub fn on_ground(&self) -> bool {
        self.grounded.on_ground()
    }
}
impl Default for GroundDetection {
    fn default() -> Self {
        Self {
            grounded: Grounded::NotOnGround,
            was_on_ground: false,
        }
    }
}

#[derive(Clone, Component)]
pub enum Grounded {
    OnGround(Entity, GroundAttrs),
    NotOnGround,
}
impl Grounded {
    fn on_ground(&self) -> bool {
        match self {
            &Grounded::OnGround(_, _) => true,
            &Grounded::NotOnGround => false,
        }
    }
}

pub fn spawn_ground_sensor(
    mut commands: Commands,
    detect_ground_for: Query<(Entity, &Collider), Added<GroundDetection>>,
) {
    for (entity, shape) in &detect_ground_for {
        if let Some(cuboid) = shape.as_cuboid() {
            let Vec2 {
                x: half_extents_x,
                y: half_extents_y,
            } = cuboid.half_extents();

            let detector_shape = Collider::cuboid(half_extents_x / 2.0, 2.);

            let sensor_translation = Vec3::new(0., -half_extents_y, 0.);

            commands.entity(entity).with_children(|builder| {
                builder
                    .spawn_empty()
                    .insert(ActiveEvents::COLLISION_EVENTS)
                    .insert(detector_shape)
                    .insert(Sensor)
                    .insert(Transform::from_translation(sensor_translation))
                    .insert(GlobalTransform::default())
                    .insert(GroundSensor {
                        ground_detection_entity: entity,
                        intersecting_ground_entities: HashMap::new(),
                    });
            });
        }
    }
}

pub fn ground_detection(
    mut ground_sensors: Query<&mut GroundSensor>,
    mut collisions: EventReader<CollisionEvent>,
    collidables: Query<(Entity, Option<&Platform>), (With<Collider>, Without<Sensor>)>,
) {
    for collision_event in collisions.read() {
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok((_ent, opt_platform)) = collidables.get(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.insert(
                            *e1,
                            GroundAttrs {
                                is_platform: opt_platform.is_some(),
                            },
                        );
                    }
                } else if let Ok((_ent, opt_platform)) = collidables.get(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.insert(
                            *e2,
                            GroundAttrs {
                                is_platform: opt_platform.is_some(),
                            },
                        );
                    }
                }
            }
            CollisionEvent::Stopped(e1, e2, _) => {
                if collidables.contains(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.remove(e1);
                    }
                } else if collidables.contains(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.remove(e2);
                    }
                }
            }
        }
    }
}

pub fn update_on_ground(
    mut ground_detectors: Query<&mut GroundDetection>,
    ground_sensors: Query<&GroundSensor, Changed<GroundSensor>>,
) {
    for sensor in &ground_sensors {
        if let Ok(mut ground_detection) = ground_detectors.get_mut(sensor.ground_detection_entity) {
            if let Some((ground_ent, ground_attrs)) =
                sensor.intersecting_ground_entities.iter().next()
            {
                ground_detection.grounded = Grounded::OnGround(*ground_ent, ground_attrs.clone());
            } else {
                ground_detection.grounded = Grounded::NotOnGround;
            }
        }
    }
}

pub fn update_was_on_ground(mut ground_detectors: Query<&mut GroundDetection>) {
    for mut ground_detector in ground_detectors.iter_mut() {
        ground_detector.was_on_ground = ground_detector.on_ground();
    }
}

/// Handles platformer-specific physics operations, specifically ground detection.
pub struct GroundDetectionPlugin;

impl Plugin for GroundDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_ground_sensor, ground_detection, update_on_ground),
        )
        .add_systems(PostUpdate, update_was_on_ground);
    }
}
