use std::{ops::DerefMut, time::Duration};

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::dynamics::Velocity;
use leafwing_input_manager::prelude::*;

use crate::{
    actions::PlatformerAction,
    climbing::Climber,
    colliders::ColliderBundle,
    ground_detection::{GroundDetection, Grounded},
    inventory::Inventory,
    jumping::Jumper,
    platform::Platform,
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
    pub movement_state: MovementState,
    pub climber: Climber,
    pub jumper: Jumper,
    pub ground_detection: GroundDetection,
    pub coyote_timer: CoyoteTimer,

    // Build Items Component manually by using `impl From<&EntityInstance>`
    #[from_entity_instance]
    items: Inventory,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

// MOVEMENT

#[derive(Component, PartialEq, Debug, Copy, Clone)]
pub enum MovementState {
    Idling,
    Running(RunningDirection),
    Climbing(ClimbingDirection),
    Jumping,
}
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RunningDirection {
    Right,
    Left,
}
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum ClimbingDirection {
    Up,
    Down,
}
impl Default for MovementState {
    fn default() -> Self {
        MovementState::Idling
    }
}

// Player jump 2 block vertically, and jump 4 horizontally but just barely.
pub fn player_movement(
    platforms_query: Query<(Entity, &Velocity), (With<Platform>, Without<Player>)>,
    mut query: Query<
        (
            &ActionState<PlatformerAction>,
            &mut MovementState,
            &mut Velocity,
            &mut Climber,
            &mut Jumper,
            &mut CoyoteTimer,
            &GroundDetection,
        ),
        (With<Player>, Without<Platform>),
    >,
) {
    for (
        action,
        mut movement_state,
        mut velocity,
        mut climber,
        mut jumper,
        mut coyote_timer,
        ground_detection,
    ) in &mut query
    {
        let on_ground = ground_detection.on_ground();

        // if on a platform, get the platform's velocity
        let (base_x_vel, base_y_vel) = match &ground_detection.grounded {
            Grounded::OnGround(ground_ent, ground_attrs) => {
                let mut x_vel = 0.;
                let mut y_vel = 0.;
                if ground_attrs.is_platform {
                    if let Ok((_, platform_vel)) = platforms_query.get(*ground_ent) {
                        x_vel = platform_vel.linvel.x;
                        y_vel = platform_vel.linvel.y;
                    }
                }
                (x_vel, y_vel)
            }
            Grounded::NotOnGround => (0., 0.),
        };

        let mut next_movement_state = movement_state.clone();

        // handle running

        let pressed_right = action.pressed(&PlatformerAction::Right);
        let pressed_left = action.pressed(&PlatformerAction::Left);
        if pressed_right && !pressed_left {
            velocity.linvel.x = base_x_vel + 150.;
            next_movement_state = MovementState::Running(RunningDirection::Right);
        } else if pressed_left && !pressed_right {
            velocity.linvel.x = base_x_vel + -150.;
            next_movement_state = MovementState::Running(RunningDirection::Left);
        } else {
            velocity.linvel.x = base_x_vel;
        }

        // handle climbing

        let just_pressed_up_or_down = action.just_pressed(&PlatformerAction::Up)
            || action.just_pressed(&PlatformerAction::Down);
        if climber.intersecting_climbables.is_empty() {
            climber.climbing = false;
        } else if just_pressed_up_or_down {
            // hitting this branch also means that the player is, in fact,
            // intersecting something climbable
            climber.climbing = true;
        }

        if climber.climbing {
            let pressed_up = action.pressed(&PlatformerAction::Up);
            let pressed_down = action.pressed(&PlatformerAction::Down);

            if pressed_up && !pressed_down {
                velocity.linvel.y = 150.;
                next_movement_state = MovementState::Climbing(ClimbingDirection::Up);
            } else if pressed_down && !pressed_up {
                velocity.linvel.y = -150.;
                next_movement_state = MovementState::Climbing(ClimbingDirection::Down);
            } else {
                velocity.linvel.y = 0.;
            }
        }

        // handle jumping
        let just_pressed_jump = action.just_pressed(&PlatformerAction::Jump);
        if just_pressed_jump {
            let is_coyote_timer_stopped = coyote_timer.0.finished() || coyote_timer.0.paused();
            match jumper.deref_mut() {
                Jumper::NotJumping => {
                    if on_ground || climber.climbing || !is_coyote_timer_stopped {
                        coyote_timer.0.pause();
                        if velocity.linvel.x == base_x_vel {
                            velocity.linvel.y = base_y_vel + 400.;
                        } else {
                            velocity.linvel.y = base_y_vel + 390.;
                        }
                        *jumper = Jumper::mk_jumping();
                        climber.climbing = false;
                        next_movement_state = MovementState::Jumping;
                    }
                }
                Jumper::Jumping(ref mut jumping) => {
                    if !jumping.double_jumping && !climber.climbing {
                        jumping.double_jumping = true;
                        velocity.linvel.y = base_y_vel + 400.;
                    }
                }
            }
        }

        if !just_pressed_jump && velocity.linvel.y == base_y_vel && on_ground {
            *jumper = Jumper::mk_not_jumping();
        }

        if !on_ground && ground_detection.was_on_ground && !jumper.is_jumping() {
            println!("set coyote");
            coyote_timer.0.reset();
            coyote_timer.0.unpause();
        }

        // set state
        if next_movement_state != *movement_state {
            *movement_state = next_movement_state;
        } else if !pressed_left
            && !pressed_left
            && !just_pressed_jump
            && !climber.climbing
            && !jumper.is_jumping()
            && velocity.linvel.x == base_x_vel
            && velocity.linvel.y == base_y_vel
            && *movement_state != MovementState::Idling
        {
            *movement_state = MovementState::Idling;
        }
    }
}

// ACTIONS

fn setup_player_actions(mut commands: Commands, mut query: Query<Entity, Added<Player>>) {
    if query.is_empty() {
        return;
    }
    let player_ent = query.single_mut();
    if let Some(mut ent_cmds) = commands.get_entity(player_ent) {
        let input_map = InputMap::new([
            (PlatformerAction::Jump, KeyCode::Space),
            (PlatformerAction::Right, KeyCode::ArrowRight),
            (PlatformerAction::Left, KeyCode::ArrowLeft),
            (PlatformerAction::Up, KeyCode::ArrowUp),
            (PlatformerAction::Down, KeyCode::ArrowDown),
        ]);
        ent_cmds.insert(InputManagerBundle::with_map(input_map));
    }
}

// SPRITE ANIMATION

#[derive(Component, PartialEq)]
struct AnimationConfig {
    first_sprite_index: usize,
    skip_sprite_indexes: Vec<usize>,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
    frame_timer_mode: TimerMode,
    for_state: MovementState,
}

impl AnimationConfig {
    fn new(
        first: usize,
        skip: Vec<usize>,
        last: usize,
        fps: u8,
        timer_mode: TimerMode,
        for_state: MovementState,
    ) -> Self {
        Self {
            first_sprite_index: first,
            skip_sprite_indexes: skip,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps, timer_mode),
            frame_timer_mode: timer_mode,
            for_state,
        }
    }

    fn timer_from_fps(fps: u8, timer_mode: TimerMode) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), timer_mode)
    }
}

fn set_sprite_animation(
    mut commands: Commands,
    mut query: Query<
        (Entity, &MovementState, Option<&mut AnimationConfig>),
        (Changed<MovementState>, With<Player>),
    >,
) {
    for (ent, movement_state, mut opt_animation) in query.iter_mut() {
        if let Some(mut ent_cmds) = commands.get_entity(ent) {
            let next_animation = get_anmation_for_movement_state(&movement_state);
            if let Some(animation) = &mut opt_animation {
                **animation = next_animation;
            } else {
                ent_cmds.insert(next_animation);
            }
        }
    }
}

fn get_anmation_for_movement_state(state: &MovementState) -> AnimationConfig {
    match state {
        MovementState::Idling | MovementState::Climbing(_) => {
            AnimationConfig::new(0, vec![1, 2, 3, 4], 5, 3, TimerMode::Repeating, *state)
        }
        MovementState::Jumping => AnimationConfig::new(4, vec![], 4, 10, TimerMode::Once, *state),
        MovementState::Running(_) => {
            AnimationConfig::new(1, vec![2], 3, 15, TimerMode::Repeating, *state)
        }
    }
}

fn animate_sprite(time: Res<Time>, mut query: Query<(&mut AnimationConfig, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut query {
        animation.frame_timer.tick(time.delta());
        if animation.frame_timer.just_finished() {
            if let Some(atlas) = &mut sprite.texture_atlas {
                let mut next_index = atlas.index + 1;
                while animation.skip_sprite_indexes.contains(&next_index) {
                    next_index = next_index + 1;
                }
                if next_index > animation.last_sprite_index {
                    atlas.index = animation.first_sprite_index
                } else {
                    atlas.index = next_index;
                    animation.frame_timer =
                        AnimationConfig::timer_from_fps(animation.fps, animation.frame_timer_mode);
                };
            }
        }
    }
}

// the sprite is flipped before the animation ends
fn set_sprite_direction(
    mut query: Query<(&mut Sprite, &AnimationConfig), (With<Player>, Changed<AnimationConfig>)>,
) {
    if query.is_empty() {
        return;
    }

    let (mut player_sprite, animation) = query.single_mut();
    match animation.for_state {
        MovementState::Running(RunningDirection::Right) => player_sprite.flip_x = false,
        MovementState::Running(RunningDirection::Left) => player_sprite.flip_x = true,
        _ => (),
    }
}

// COYOTE TIMER

#[derive(Component, Clone)]
pub struct CoyoteTimer(Timer);

impl Default for CoyoteTimer {
    fn default() -> Self {
        let mut coyote_timer = Timer::new(Duration::from_secs_f32(0.2), TimerMode::Once);
        coyote_timer.pause();
        Self(coyote_timer)
    }
}

fn tick_timers(time: Res<Time>, mut query: Query<&mut CoyoteTimer>) {
    for mut coyote in query.iter_mut() {
        coyote.0.tick(time.delta());
    }
}

// PLUGIN

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.register_ldtk_entity::<PlayerBundle>("Player")
            .add_systems(Update, (setup_player_actions, player_movement, tick_timers))
            .add_systems(
                Update,
                (
                    set_sprite_animation,
                    set_sprite_direction.after(set_sprite_animation),
                    animate_sprite,
                ),
            );
    }
}
