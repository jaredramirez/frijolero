use std::{ops::DerefMut, time::Duration};

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::dynamics::Velocity;
use leafwing_input_manager::prelude::*;

use crate::jumping::{DashDirection, Dasher};
use crate::timer_helpers::TimerHelper;
use crate::{
    actions::PlatformerAction,
    climbing::Climber,
    colliders::ColliderBundle,
    ground_detection::{CoyoteTimer, GroundDetection},
    inventory::Inventory,
    jumping::Jumper,
    platform::Platform,
};

/// tag for players
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Player;

/// player bundle, containing everything needed
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
    pub jumper: Jumper,
    pub dasher: Dasher,
    pub ground_detection: GroundDetection,
    pub coyote_timer: CoyoteTimer,
    pub jump_buffer_timer: JumpBufferTimer,

    // Build Items Component manually by using `impl From<&EntityInstance>`
    #[from_entity_instance]
    items: Inventory,

    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    entity_instance: EntityInstance,
}

// MOVEMENT

// movement constants

const JUMP_VELOCITY: f32 = 400.;
const RUN_VELOCITY: f32 = 150.;
const CLIMB_VELOCITY: f32 = 150.;
const DASH_VELOCITY: f32 = 600.;
const DASH_VELOCITY_VEC: Vec2 = Vec2::new(DASH_VELOCITY, DASH_VELOCITY);

/// configure player movement
pub fn player_movement(
    mut animation_event: EventWriter<AnimationEvent>,
    platforms_query: Query<(Entity, &Velocity), (With<Platform>, Without<Player>)>,
    mut player_query: Query<
        (
            Entity,
            &Sprite,
            &ActionState<PlatformerAction>,
            &mut Velocity,
            &mut Climber,
            &mut Jumper,
            &mut Dasher,
            &mut CoyoteTimer,
            &mut JumpBufferTimer,
            &GroundDetection,
        ),
        (With<Player>, Without<Platform>),
    >,
) {
    for (
        ent,
        sprite,
        action,
        mut velocity,
        mut climber,
        mut jumper,
        mut dasher,
        mut coyote_timer,
        mut jump_buffer_timer,
        ground_detection,
    ) in &mut player_query
    {
        let on_ground = ground_detection.on_ground();

        // if on a platform, get the platform's velocity
        // this is the base velocity on top of any user input movement velocity
        let base_vel = match &ground_detection {
            GroundDetection::OnGround(ground_ent) => {
                if let Ok((_, platform_vel)) = platforms_query.get(*ground_ent) {
                    platform_vel.linvel
                } else {
                    Vec2::new(0., 0.)
                }
            }
            GroundDetection::NotOnGround => Vec2::new(0., 0.),
        };

        // handle running

        // see if the player just pressed right/left
        let pressed_right = action.pressed(&PlatformerAction::Right);
        let pressed_left = action.pressed(&PlatformerAction::Left);

        // set x velocity
        if pressed_right && !pressed_left {
            velocity.linvel.x = base_vel.x + RUN_VELOCITY;
            animation_event.send(AnimationEvent::running(ent, RunningDirection::Right));
        } else if pressed_left && !pressed_right {
            velocity.linvel.x = base_vel.x + -RUN_VELOCITY;
            animation_event.send(AnimationEvent::running(ent, RunningDirection::Left));
        } else {
            velocity.linvel.x = base_vel.x;
        }

        // handle climbing

        // see if the player just pressed up/down
        let just_pressed_up_or_down = action.just_pressed(&PlatformerAction::Up)
            || action.just_pressed(&PlatformerAction::Down);

        // set climbing state
        if climber.intersecting_climbables.is_empty() {
            // if the climber isn't intersecting a climbable, then we're def not
            // climbing
            climber.climbing = false;
        } else if just_pressed_up_or_down {
            // ^ implied && !climber.intersecting_climbables.is_empty()
            // if the climber intersecting a climbable and just pressed up/down
            // then we are climbing
            climber.climbing = true;
        }

        // if we're climbing and we're pressing up/down, set out velocity
        let pressed_up = action.pressed(&PlatformerAction::Up);
        let pressed_down = action.pressed(&PlatformerAction::Down);
        if climber.climbing {
            if pressed_up && !pressed_down {
                velocity.linvel.y = CLIMB_VELOCITY;
                animation_event.send(AnimationEvent::climbing(ent, ClimbingDirection::Up));
            } else if pressed_down && !pressed_up {
                velocity.linvel.y = -CLIMB_VELOCITY;
                animation_event.send(AnimationEvent::climbing(ent, ClimbingDirection::Down));
            } else {
                velocity.linvel.y = 0.;
            }
        }

        // handle the jump buffer

        // if we're on the ground and the jump buffer is running, that means
        // the user pressed jump in the air recently
        if on_ground && !jump_buffer_timer.0.is_stopped() {
            jump_buffer_timer.0.pause();
            velocity.linvel.y = base_vel.y + JUMP_VELOCITY;
            *jumper = Jumper::mk_jumping();
        }

        // handle jumping

        // see if we just pressed jump
        let just_pressed_jump = action.just_pressed(&PlatformerAction::Jump);

        // if you pressed jump
        if just_pressed_jump {
            match jumper.deref_mut() {
                // and you're _not_ currently jumping and you're on the ground,
                // climbing, or we're within range of the coyote timer, then
                // jump
                Jumper::NotJumping
                    if on_ground || climber.climbing || !coyote_timer.0.is_stopped() =>
                {
                    // disable the coyote timer (may be noop)
                    coyote_timer.0.pause();

                    // set the y vel
                    velocity.linvel.y = base_vel.y + JUMP_VELOCITY;

                    // set game state
                    *jumper = Jumper::mk_jumping();
                    climber.climbing = false;
                    animation_event.send(AnimationEvent::jumping(ent));
                }
                // and are mid-jump, and are not climbing
                Jumper::Jumping(ref mut jumping) if !climber.climbing => {
                    // see if you have any jumps left, and if so decrement
                    // your remaining jumps
                    if jumping.jumps_left > 0 {
                        velocity.linvel.y = base_vel.y + JUMP_VELOCITY;
                        jumping.jumps_left -= 1;
                    } else {
                        // trigger the jump buffer
                        jump_buffer_timer.0.restart();
                    }
                }
                _ => {}
            }
        }

        // handle dashing

        // see if we just pressed dash
        let just_pressed_dash = action.just_pressed(&PlatformerAction::Dash);

        // if you pressed dash
        if just_pressed_dash {
            let dash_dir = DashDirection::make(
                pressed_up,
                pressed_right,
                pressed_down,
                pressed_left,
                sprite.flip_x,
            );
            let dash_dir_vec = dash_dir.to_vec() * DASH_VELOCITY_VEC;
            let next_linevel = velocity.linvel + dash_dir_vec + base_vel;
            println!("{:?} {} {}", dash_dir, dash_dir_vec, next_linevel);

            match dasher.deref_mut() {
                // and you're _not_ currently dashing or climbing, then dash
                Dasher::NotDashing if !climber.climbing => {
                    // set the y vel
                    velocity.linvel = next_linevel;

                    // set game state
                    *dasher = Dasher::mk_dashing();
                    animation_event.send(AnimationEvent::dashing(ent));
                }
                // and you currently dashing with more dashes left
                Dasher::Dashing(ref mut dashing) if !climber.climbing && dashing.dashs_left > 0 => {
                    // dash, then decrement your remaining dashs
                    velocity.linvel = next_linevel;
                    dashing.dashs_left -= 1;
                }
                _ => {}
            }
        }

        // If you didn't just just press jump, you y vel is stable, and you're
        // on the ground, then reset jump state
        if !just_pressed_jump && velocity.linvel.y == base_vel.y && on_ground {
            *jumper = Jumper::mk_not_jumping();
        }

        // If you didn't just just press dash, you y vel is stable, and you're
        // on the ground, then reset dash state
        if !just_pressed_dash
            && velocity.linvel.y == base_vel.y
            && velocity.linvel.x == base_vel.x
            && on_ground
        {
            *dasher = Dasher::mk_not_dashing();
        }

        // set movement state
        if !pressed_left
            && !pressed_right
            && !just_pressed_jump
            && !climber.climbing
            && !jumper.is_jumping()
            && velocity.linvel.x == base_vel.x
            && velocity.linvel.y == base_vel.y
        {
            animation_event.send(AnimationEvent::idling(ent));
        }
    }
}

// ACTIONS

/// configure the keys -> action mapping  for the player
fn setup_player_actions(mut commands: Commands, mut query: Query<Entity, Added<Player>>) {
    if query.is_empty() {
        return;
    }
    let player_ent = query.single_mut();
    if let Some(mut ent_cmds) = commands.get_entity(player_ent) {
        let input_map = InputMap::new([
            (PlatformerAction::Right, KeyCode::ArrowRight),
            (PlatformerAction::Left, KeyCode::ArrowLeft),
            (PlatformerAction::Up, KeyCode::ArrowUp),
            (PlatformerAction::Down, KeyCode::ArrowDown),
            (PlatformerAction::Jump, KeyCode::Space),
            (PlatformerAction::Jump, KeyCode::KeyZ),
            (PlatformerAction::Dash, KeyCode::KeyX),
        ]);
        ent_cmds.insert(InputManagerBundle::with_map(input_map));
    }
}

// SPRITE ANIMATION

#[derive(Event, PartialEq, Debug, Copy, Clone)]
pub struct AnimationEvent {
    ent: Entity,
    typ: AnimationEventType,
}
impl AnimationEvent {
    fn idling(ent: Entity) -> Self {
        AnimationEvent {
            ent,
            typ: AnimationEventType::Idling,
        }
    }
    fn running(ent: Entity, dir: RunningDirection) -> Self {
        AnimationEvent {
            ent,
            typ: AnimationEventType::Running(dir),
        }
    }
    fn climbing(ent: Entity, dir: ClimbingDirection) -> Self {
        AnimationEvent {
            ent,
            typ: AnimationEventType::Climbing(dir),
        }
    }
    fn jumping(ent: Entity) -> Self {
        AnimationEvent {
            ent,
            typ: AnimationEventType::Jumping,
        }
    }
    fn dashing(ent: Entity) -> Self {
        AnimationEvent {
            ent,
            typ: AnimationEventType::Dashing,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum AnimationEventType {
    Idling,
    Running(RunningDirection),
    Climbing(ClimbingDirection),
    Jumping,
    Dashing,
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

// animating config
#[derive(Component, PartialEq, Debug)]
struct AnimationConfig {
    first_sprite_index: usize,
    skip_sprite_indexes: Vec<usize>,
    last_sprite_index: usize,
    fps: u8,
    frame_timer: Timer,
    frame_timer_mode: TimerMode,
    for_event: AnimationEventType,
}
impl AnimationConfig {
    fn new(
        first: usize,
        skip: Vec<usize>,
        last: usize,
        fps: u8,
        timer_mode: TimerMode,
        for_event: AnimationEventType,
    ) -> Self {
        Self {
            first_sprite_index: first,
            skip_sprite_indexes: skip,
            last_sprite_index: last,
            fps,
            frame_timer: Self::timer_from_fps(fps, timer_mode),
            frame_timer_mode: timer_mode,
            for_event,
        }
    }

    fn timer_from_fps(fps: u8, timer_mode: TimerMode) -> Timer {
        Timer::new(Duration::from_secs_f32(1.0 / (fps as f32)), timer_mode)
    }
}

/// set the sprite animation for player
fn recieve_animation_event(
    mut commands: Commands,
    mut animation_events: EventReader<AnimationEvent>,
    mut animation_config_query: Query<&mut AnimationConfig, With<Player>>,
) {
    for animation_event in animation_events.read() {
        let next_animation = get_anmation_for_movement_event(&animation_event.typ);
        if let Ok(mut animation) = animation_config_query.get_mut(animation_event.ent) {
            if animation.for_event != next_animation.for_event {
                *animation = next_animation;
            }
        } else {
            if let Some(mut ent_cmds) = commands.get_entity(animation_event.ent) {
                ent_cmds.insert(next_animation);
            }
        }
    }
}

/// for the provided movement state, get the animation config
fn get_anmation_for_movement_event(event_type: &AnimationEventType) -> AnimationConfig {
    match event_type {
        AnimationEventType::Idling | AnimationEventType::Climbing(_) => {
            AnimationConfig::new(0, vec![1, 2, 3, 4], 5, 3, TimerMode::Repeating, *event_type)
        }
        AnimationEventType::Jumping | AnimationEventType::Dashing => {
            AnimationConfig::new(4, vec![], 4, 10, TimerMode::Once, *event_type)
        }
        AnimationEventType::Running(_) => {
            AnimationConfig::new(1, vec![2], 3, 15, TimerMode::Repeating, *event_type)
        }
    }
}

/// animate the sprite with the current animation config
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

/// flip the sprite animation based on the players movement direction
fn set_sprite_direction(
    mut query: Query<(&mut Sprite, &AnimationConfig), (With<Player>, Changed<AnimationConfig>)>,
) {
    if query.is_empty() {
        return;
    }

    let (mut sprite, animation) = query.single_mut();
    match animation.for_event {
        AnimationEventType::Running(RunningDirection::Right) => sprite.flip_x = false,
        AnimationEventType::Running(RunningDirection::Left) => sprite.flip_x = true,
        _ => (),
    }
}

// JUMP BUFFR TIMER

/// store the jump buffer
#[derive(Component, Clone)]
pub struct JumpBufferTimer(Timer);

impl Default for JumpBufferTimer {
    fn default() -> Self {
        let mut jump_buffer_timer = Timer::new(Duration::from_secs_f32(0.1), TimerMode::Once);
        jump_buffer_timer.pause();
        Self(jump_buffer_timer)
    }
}

/// tick the jump buffer
fn tick_jump_buffer(time: Res<Time>, mut query: Query<&mut JumpBufferTimer>) {
    for mut jump_buffer in query.iter_mut() {
        jump_buffer.0.tick(time.delta());
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
                // player movement systems
                (setup_player_actions, player_movement, tick_jump_buffer),
            )
            .add_systems(
                Update,
                // sprite systems
                (
                    recieve_animation_event,
                    set_sprite_direction.after(recieve_animation_event),
                    animate_sprite,
                ),
            );
    }
}
