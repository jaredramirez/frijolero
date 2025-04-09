use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::math::Vect;
use bevy_rapier2d::prelude::{CoefficientCombineRule, Collider, Friction, RigidBody};
use bevy_tnua::builtins::TnuaBuiltinJumpState;
use bevy_tnua::math::Vector3;
use bevy_tnua::prelude::{TnuaBuiltinJump, TnuaBuiltinWalk, TnuaController};
use bevy_tnua::{TnuaAction, TnuaAnimatingState, TnuaAnimatingStateDirective};
use bevy_tnua_rapier2d::TnuaRapier2dIOBundle;
use bevy_tnua_rapier2d::TnuaRapier2dSensorShape;
use leafwing_input_manager::prelude::*;

use crate::colliders::ROTATION_CONSTRAINTS;
use crate::game_flow::{RespawnLevelEvent, RespawnWorldEvent};
use crate::spike::SpikeDetection;
use crate::{actions::PlatformerAction, colliders::ColliderBundle};

/// tag for players
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

/// player bundle, containing everything needed
#[derive(Default, Bundle, LdtkEntity)]
pub struct PlayerBundle {
    pub player: Player,
    pub movement_direction: MovementDirection,
    pub spike_detection: SpikeDetection,
    pub air_jumps_left: AirJumpsLeft,

    #[sprite_sheet("player.png", 16, 16, 7, 1, 0, 0, 0)]
    pub sprite: Sprite,

    #[worldly]
    pub worldly: Worldly,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

/// track the direction the player is facing
#[derive(Component, PartialEq, Debug, Copy, Clone, Default)]
pub enum MovementDirection {
    #[default]
    None,
    Right,
    Left,
}

/// track the direction the player is facing
#[derive(Component, PartialEq, Debug, Copy, Clone)]
pub struct AirJumpsLeft {
    num: u8,
}
impl Default for AirJumpsLeft {
    fn default() -> Self {
        AirJumpsLeft { num: 2 }
    }
}

// MOVEMENT

// movement constants

pub fn player_movement(
    mut player_query: Query<
        (
            &ActionState<PlatformerAction>,
            &mut TnuaController,
            &mut MovementDirection,
            &mut AirJumpsLeft,
        ),
        With<Player>,
    >,
) {
    for (action, mut tnua_controller, mut movement_dir, mut air_jumps_left) in &mut player_query {
        let mut direction = Vector3::ZERO;

        // see if the player just pressed right/left
        let pressed_left = action.pressed(&PlatformerAction::Left);
        if pressed_left {
            direction -= Vector3::X;
            *movement_dir = MovementDirection::Left;
        }

        let pressed_right = action.pressed(&PlatformerAction::Right);
        if pressed_right {
            direction += Vector3::X;
            *movement_dir = MovementDirection::Right;
        }

        if !pressed_left && !pressed_right {
            *movement_dir = MovementDirection::None;
        }

        direction = direction.clamp_length_max(1.0);

        tnua_controller.basis(TnuaBuiltinWalk {
            desired_velocity: direction * 200.,
            air_acceleration: 800.,
            acceleration: 800.,
            float_height: 4.8,
            ..Default::default()
        });

        let just_pressed_jump = action.just_pressed(&PlatformerAction::Jump);
        dbg!(air_jumps_left.num);
        if just_pressed_jump && air_jumps_left.num > 0 {
            air_jumps_left.num -= 1;
        }

        let pressed_jump = action.pressed(&PlatformerAction::Jump);
        if pressed_jump {
            tnua_controller.action(TnuaBuiltinJump {
                height: 35.0,
                allow_in_air: air_jumps_left.num > 0,
                ..Default::default()
            });
        }

        if let Ok(false) = tnua_controller.is_airborne() {
            *air_jumps_left = AirJumpsLeft::default();
        }
    }
}

// ACTIONS

/// configure the keys -> action mapping  for the player
fn setup_player(mut commands: Commands, mut query: Query<Entity, Added<Player>>) {
    if query.is_empty() {
        return;
    }
    let player_ent = query.single_mut();
    if let Some(mut ent_cmds) = commands.get_entity(player_ent) {
        // Setup the player keymap
        let player_input_map = InputMap::new([
            (PlatformerAction::Jump, KeyCode::Space),
            (PlatformerAction::Right, KeyCode::ArrowRight),
            (PlatformerAction::Left, KeyCode::ArrowLeft),
            (PlatformerAction::Up, KeyCode::ArrowUp),
            (PlatformerAction::Down, KeyCode::ArrowDown),
            (PlatformerAction::RespawnLevel, KeyCode::KeyR),
            (PlatformerAction::RespawnWorld, KeyCode::KeyG),
        ]);
        ent_cmds.insert(InputManagerBundle::with_map(player_input_map));

        // Setup collider
        ent_cmds.insert(ColliderBundle {
            collider: Collider::compound(vec![
                (Vect::new(0., 2.), 0., Collider::cuboid(6., 2.)),
                (Vect::new(0., -4.), 0., Collider::cuboid(6., 4.)),
            ]),
            rigid_body: RigidBody::Dynamic,
            friction: Friction {
                coefficient: 0.,
                combine_rule: CoefficientCombineRule::Min,
            },
            rotation_constraints: ROTATION_CONSTRAINTS,
            ..Default::default()
        });

        // Setup tnua
        ent_cmds.insert(TnuaRapier2dIOBundle::default());
        ent_cmds.insert(TnuaController::default());
        ent_cmds.insert(TnuaRapier2dSensorShape(Collider::cuboid(5.75, 4.)));
        ent_cmds.insert(TnuaAnimatingState::<AnimationState>::default());
    }
}

/// configure the keys -> action mapping  for the player
fn handle_game_actions(
    mut level_respawn_event: EventWriter<RespawnLevelEvent>,
    mut world_respawn_event: EventWriter<RespawnWorldEvent>,
    query: Query<&ActionState<PlatformerAction>, With<Player>>,
) {
    for action in query.iter() {
        if action.just_pressed(&PlatformerAction::RespawnLevel) {
            level_respawn_event.send(RespawnLevelEvent::RespawnLevelEvent);
        } else if action.just_pressed(&PlatformerAction::RespawnWorld) {
            world_respawn_event.send(RespawnWorldEvent::RespawnWorldEvent);
        }
    }
}

// SPRITE ANIMATION

#[allow(clippy::unnecessary_cast)]
pub fn animate(
    mut animations_handlers_query: Query<(
        Entity,
        &mut TnuaAnimatingState<AnimationState>,
        &TnuaController,
    )>,
    mut animation_event: EventWriter<AnimationEvent>,
) {
    for (ent, mut animating_state, controller) in animations_handlers_query.iter_mut() {
        let current_status_for_animating = match controller.action_name() {
            Some(TnuaBuiltinJump::NAME) => {
                let (_, jump_state) = controller
                    .concrete_action::<TnuaBuiltinJump>()
                    .expect("action name mismatch");

                // Depending on the state of the jump, we need to decide if we want to play the jump
                // animation or the fall animation.
                match jump_state {
                    TnuaBuiltinJumpState::NoJump => return,
                    TnuaBuiltinJumpState::StartingJump { .. } => AnimationState::Jumping,
                    TnuaBuiltinJumpState::SlowDownTooFastSlopeJump { .. } => {
                        AnimationState::Jumping
                    }
                    TnuaBuiltinJumpState::MaintainingJump => AnimationState::Jumping,
                    TnuaBuiltinJumpState::StoppedMaintainingJump => AnimationState::Jumping,
                    TnuaBuiltinJumpState::FallSection => AnimationState::Falling,
                }
            }
            Some(other) => panic!("Unknown action {other}"),
            None => {
                let Some((_, basis_state)) = controller.concrete_basis::<TnuaBuiltinWalk>() else {
                    return;
                };

                if basis_state.standing_on_entity().is_none() {
                    AnimationState::Falling
                } else {
                    let speed = basis_state.running_velocity.length();
                    if 0.01 < speed {
                        AnimationState::Running
                    } else {
                        AnimationState::Standing
                    }
                }
            }
        };

        let animating_directive =
            animating_state.update_by_discriminant(current_status_for_animating);

        match animating_directive {
            TnuaAnimatingStateDirective::Maintain { state: _ } => {}
            TnuaAnimatingStateDirective::Alter {
                old_state: _,
                state,
            } => {
                animation_event.send(AnimationEvent { ent, state: *state });
            }
        };
    }
}

// SPRITE ANIMATION

/// animating info
struct AnimationInfo {
    sprite_indexes: &'static [usize],
    fps: u8,
    frame_timer_mode: TimerMode,
}
impl AnimationInfo {
    fn from_state(state: &AnimationState) -> Self {
        match state {
            &AnimationState::Standing => ANIMATION_INFO_STANDING,
            &AnimationState::Jumping => ANIMATION_INFO_JUMPING,
            &AnimationState::Falling => ANIMATION_INFO_JUMPING,
            &AnimationState::Running => ANIMATION_INFO_RUNNING,
        }
    }
}

// ANIMATING INFO CONSTANTS

const ANIMATION_INFO_STANDING: AnimationInfo = AnimationInfo {
    sprite_indexes: &[0, 3],
    fps: 2,
    frame_timer_mode: TimerMode::Repeating,
};
const ANIMATION_INFO_JUMPING: AnimationInfo = AnimationInfo {
    sprite_indexes: &[2],
    fps: 10,
    frame_timer_mode: TimerMode::Repeating,
};
const ANIMATION_INFO_RUNNING: AnimationInfo = AnimationInfo {
    sprite_indexes: &[1, 4],
    fps: 15,
    frame_timer_mode: TimerMode::Repeating,
};

// ANIMATION CONFIG

#[derive(Component)]
struct AnimationConfig {
    state: AnimationState,
    frame_timer: Timer,
}
impl AnimationConfig {
    fn mk(info: &AnimationInfo, state: AnimationState) -> AnimationConfig {
        AnimationConfig {
            state,
            frame_timer: Self::timer_from_fps(info.fps, info.frame_timer_mode),
        }
    }

    fn timer_from_fps(fps: u8, timer_mode: TimerMode) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), timer_mode)
    }
}

// SPRITE ANIMATION

#[derive(Event, PartialEq, Debug, Copy, Clone)]
pub struct AnimationEvent {
    ent: Entity,
    state: AnimationState,
}

#[derive(PartialEq, Debug, Copy, Clone, Eq, Hash)]
pub enum AnimationState {
    Standing,
    Running,
    Jumping,
    Falling,
}

/// set the sprite animation for player
fn recieve_animation_event(
    mut commands: Commands,
    mut animation_events: EventReader<AnimationEvent>,
    mut animation_config_query: Query<&mut AnimationConfig, With<Player>>,
) {
    for animation_event in animation_events.read() {
        let new_info = AnimationInfo::from_state(&animation_event.state);
        let new_config = AnimationConfig::mk(&new_info, animation_event.state);

        if let Ok(mut existing) = animation_config_query.get_mut(animation_event.ent) {
            if existing.state != animation_event.state {
                *existing = new_config;
            }
        } else {
            commands.entity(animation_event.ent).insert(new_config);
        }
    }
}

/// animate the sprite with the current animation config
fn animate_sprite(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut query {
        let info = AnimationInfo::from_state(&animation.state);

        animation.frame_timer.tick(time.delta());
        if animation.frame_timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                let opt_index_of_atlas = info.sprite_indexes.iter().position(|v| v == &atlas.index);
                if let Some(index_of_atlas) = opt_index_of_atlas {
                    if let Some(next_index) = info.sprite_indexes.get(index_of_atlas + 1) {
                        atlas.index = *next_index;
                    } else if let Some(first_index) = info.sprite_indexes.get(0) {
                        atlas.index = *first_index;
                    }
                } else {
                    if let Some(first_index) = info.sprite_indexes.get(0) {
                        atlas.index = *first_index;
                    }
                }
            }
        }
    }
}

/// flip the sprite animation based on the players movement direction
fn set_sprite_direction(
    mut query: Query<(&mut Sprite, &MovementDirection), (With<Player>, Changed<MovementDirection>)>,
) {
    if query.is_empty() {
        return;
    }

    let (mut sprite, movement_dir) = query.single_mut();
    match movement_dir {
        MovementDirection::Right => sprite.flip_x = false,
        MovementDirection::Left => sprite.flip_x = true,
        MovementDirection::None => (),
    }
}

// PLUGIN

/// handles player movement & sprite anmiation
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AnimationEvent>()
            .register_ldtk_entity::<PlayerBundle>("Player")
            .add_systems(
                Update,
                (
                    setup_player,
                    handle_game_actions,
                    player_movement,
                    animate,
                    recieve_animation_event,
                    animate_sprite,
                    set_sprite_direction,
                ),
            );
    }
}
