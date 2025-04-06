use std::{env, path::Path};

use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use bevy_tnua::prelude::TnuaControllerPlugin;
use bevy_tnua_rapier2d::TnuaRapier2dPlugin;
use clap::Parser;
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
mod spike;
mod timer_helpers;
mod walls;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    ldtk_file: Option<String>,

    #[arg(short, long, default_value_t = false)]
    dev: bool,
}

fn main() {
    let args = Args::parse();

    let game_path_string = args.ldtk_file.unwrap_or("bean_platformer.ldtk".to_string());
    let game_path = Path::new(&game_path_string).to_path_buf();

    let is_in_dev_mode = args.dev;

    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    // TODO(prod): Disable on prod
                    watch_for_changes_override: None,
                    ..Default::default()
                }),
        )
        .add_plugins(TnuaControllerPlugin::default())
        .add_plugins(TnuaRapier2dPlugin::default())
        .add_plugins(InputManagerPlugin::<actions::PlatformerAction>::default())
        .add_plugins(LdtkPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
        .add_plugins(DevPlugin { is_in_dev_mode })
        .insert_resource(GameFile { path: game_path })
        .insert_resource(LevelSelection::Uid(0))
        .insert_resource(LdtkSettings {
            level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
                load_level_neighbors: true,
            },
            set_clear_color: SetClearColor::FromLevelBackground,
            ..Default::default()
        })
        .add_event::<game_flow::RespawnWorldEvent>()
        .add_event::<game_flow::RespawnLevelEvent>()
        .add_plugins(game_flow::GameFlowPlugin)
        .add_plugins(player::PlayerPlugin)
        .add_plugins(ground_detection::GroundDetectionPlugin)
        .add_plugins(walls::WallsPlugin)
        .add_plugins(climbing::ClimbingPlugin)
        .add_plugins(enemy::EnemyPlugin)
        .add_plugins(obstacle::ObstaclePlugin)
        .add_plugins(platform::PlatformPlugin)
        .add_plugins(spike::SpikePlugin)
        .add_plugins(misc_objects::MiscObjectsPlugin)
        .add_systems(Update, inventory::dbg_print_inventory)
        .add_systems(Update, camera::camera_fit_inside_current_level)
        .run();
}

struct DevPlugin {
    is_in_dev_mode: bool,
}

impl Plugin for DevPlugin {
    fn build(&self, app: &mut App) {
        if self.is_in_dev_mode {
            app.add_plugins(RapierDebugRenderPlugin::default());
        }
    }
}
