use std::{env, path::Path};

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use game_flow::GameFile;
use leafwing_input_manager::prelude::*;

mod actions;
mod camera;
mod climbing;
mod colliders;
mod enemy;
mod game_flow;
mod ground_detection;
mod inventory;
mod jumping;
mod misc_objects;
mod obstacle;
mod platform;
mod player;
mod timer_helpers;
mod walls;

fn main() {
    let game_path_string: String = env::args()
        .nth(1)
        .unwrap_or("bean_platformer.ldtk".to_string());
    let game_path = Path::new(&game_path_string).to_path_buf();

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin::default()),
        )
        .add_plugins(InputManagerPlugin::<actions::PlatformerAction>::default())
        .add_plugins((
            LdtkPlugin,
            RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0),
            RapierDebugRenderPlugin::default(),
        ))
        .insert_resource(GameFile { path: game_path })
        .insert_resource(LevelSelection::Uid(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .add_plugins(game_flow::GameFlowPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(ground_detection::GroundDetectionPlugin)
        .add_plugins(walls::WallsPlugin)
        .add_plugins(climbing::ClimbingPlugin)
        .add_plugins(enemy::EnemyPlugin)
        .add_plugins(obstacle::ObstaclePlugin)
        .add_plugins(platform::PlatformPlugin)
        .add_plugins(misc_objects::MiscObjectsPlugin)
        .add_systems(Update, inventory::dbg_print_inventory)
        .add_systems(Update, camera::camera_fit_inside_current_level)
        .run();
}
