use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use std::{collections::HashSet, time::Duration};

use crate::timer_helpers::TimerHelper;

// ground detection

/// component to pub on all thing we want to know their ground state
#[derive(Component)]
pub struct GroundSensor {
    pub ground_detection_entity: Entity,
    pub intersecting_ground_entities: HashSet<Entity>,
}

///
#[derive(Clone, Component, Default)]
pub enum GroundDetection {
    OnGround(Entity),
    #[default]
    NotOnGround,
}
impl GroundDetection {
    pub fn on_ground(&self) -> bool {
        match self {
            &GroundDetection::OnGround(_) => true,
            &GroundDetection::NotOnGround => false,
        }
    }
}

/// coyote timer
#[derive(Component, Clone)]
pub struct CoyoteTimer(pub Timer);
impl Default for CoyoteTimer {
    fn default() -> Self {
        let mut coyote_timer = Timer::new(Duration::from_secs_f32(0.2), TimerMode::Once);
        coyote_timer.pause();
        Self(coyote_timer)
    }
}

/// when GroundDetection is added to entity, add various other components
/// needed for ground detection. notably GroundSensor & collision events
pub fn spawn_ground_sensor(
    mut commands: Commands,
    detect_ground_for: Query<(Entity, &Collider), Added<GroundDetection>>,
) {
    for (entity, shape) in &detect_ground_for {
        // First, we get the collider cuboid shape
        let opt_cuboid_half_extents = if let Some(cuboid) = shape.as_cuboid() {
            // If the shape is a cuboid, then easy
            Some((cuboid.half_extents(), 0.))
        } else if let Some(compound) = shape.as_compound() {
            // If this is a compound shape, get the cuboid with the lowest y val
            let mut lowest_y = f32::MAX;
            let mut opt_selected = None;
            for (cur_pos, _rot, col) in compound.shapes() {
                if let ColliderView::Cuboid(cur_cuboid) = col {
                    if cur_pos.y < lowest_y {
                        opt_selected = Some((cur_cuboid.half_extents(), cur_pos.y));
                        lowest_y = cur_pos.y;
                    }
                }
            }
            opt_selected
        } else {
            None
        };

        // Insert a sensor collider at the bottom of the regulare colliaed
        if let Some((cuboid_half_extents, y_offset)) = opt_cuboid_half_extents {
            let Vec2 {
                x: half_extents_x,
                y: half_extents_y,
            } = cuboid_half_extents;
            let detector_shape = Collider::cuboid(half_extents_x / 2.0, 2.);
            let sensor_translation = Vec3::new(0., -half_extents_y + y_offset, 0.);
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
                        intersecting_ground_entities: HashSet::new(),
                    });
            });
        }
    }
}

/// update the GroundSensor every time a collision event occurs
pub fn ground_detection(
    mut ground_sensors: Query<&mut GroundSensor>,
    mut collisions: EventReader<CollisionEvent>,
    collidables: Query<Entity, (With<Collider>, Without<Sensor>)>,
) {
    for collision_event in collisions.read() {
        // if we're we have an evenr, figure out which entity is
        // intersecting which and add/remove it from our list of collision
        match collision_event {
            CollisionEvent::Started(e1, e2, _) => {
                if let Ok(_ent) = collidables.get(*e1) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e2) {
                        sensor.intersecting_ground_entities.insert(*e1);
                    }
                } else if let Ok(_ent) = collidables.get(*e2) {
                    if let Ok(mut sensor) = ground_sensors.get_mut(*e1) {
                        sensor.intersecting_ground_entities.insert(*e2);
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

/// update GroundDetection based on GroundSensor
pub fn update_on_ground(
    mut ground_detectors: Query<(&mut GroundDetection, &mut CoyoteTimer)>,
    ground_sensors: Query<&GroundSensor, Changed<GroundSensor>>,
) {
    // for every sensor
    for sensor in &ground_sensors {
        // get the ground detection & coyote time for the sensor entity
        if let Ok((mut ground_detection, mut coyote_timer)) =
            ground_detectors.get_mut(sensor.ground_detection_entity)
        {
            let old_on_ground = ground_detection.on_ground();

            // update ground detection
            if let Some(ground_ent) = sensor.intersecting_ground_entities.iter().next() {
                *ground_detection = GroundDetection::OnGround(*ground_ent);
            } else {
                *ground_detection = GroundDetection::NotOnGround;

                // if we were on the ground, but now we're not, start the
                // coyote timer
                if old_on_ground {
                    coyote_timer.0.restart();
                }
            }
        }
    }
}

/// tick the coyote timer
fn tick_coyote_timer(time: Res<Time>, mut query: Query<&mut CoyoteTimer>) {
    for mut coyote in query.iter_mut() {
        coyote.0.tick(time.delta());
    }
}

/// handles platformer-specific physics operations, specifically ground detection.
pub struct GroundDetectionPlugin;

impl Plugin for GroundDetectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_ground_sensor,
                ground_detection,
                update_on_ground,
                tick_coyote_timer,
            ),
        );
    }
}
