use std::path::PathBuf;

use bevy::{asset::AssetPath, prelude::*};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::player::Player;

#[derive(Resource)]
pub struct GameFile {
    pub path: PathBuf,
}

pub fn setup(
    mut commands: Commands,
    game_file: Res<GameFile>,
    asset_server: Res<AssetServer>,
    mut rapier_config: Query<&mut RapierConfiguration>,
) {
    commands.spawn(Camera2d);

    rapier_config.single_mut().gravity = Vec2::new(0.0, -2000.0);

    let ldtk_handle = asset_server
        .load(AssetPath::from_path(&game_file.path))
        .into();
    commands.spawn(LdtkWorldBundle {
        ldtk_handle,
        ..Default::default()
    });
}

pub fn update_level_selection(
    level_query: Query<(&LevelIid, &Transform), Without<Player>>,
    player_query: Query<&Transform, With<Player>>,
    mut level_selection: ResMut<LevelSelection>,
    ldtk_projects: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    for (level_iid, level_transform) in &level_query {
        // Load the project & level
        let ldtk_project = ldtk_project_assets
            .get(ldtk_projects.single())
            .expect("Project should be loaded if level has spawned");
        let level = ldtk_project
            .get_raw_level_by_iid(&level_iid.to_string())
            .expect("Spawned level should exist in LDtk project");

        // Get the bounds of the level
        let level_bounds = Rect {
            min: Vec2::new(level_transform.translation.x, level_transform.translation.y),
            max: Vec2::new(
                level_transform.translation.x + level.px_wid as f32,
                level_transform.translation.y + level.px_hei as f32,
            ),
        };

        // Check if the level is an iid
        let is_iid = match *level_selection {
            LevelSelection::Iid(_) => true,
            _ => false,
        };
        // Ensure that the level_selection is always an iid. This is needed by
        // the respawn logic
        if level_selection.is_match(&LevelIndices::default(), level) && !is_iid {
            *level_selection = LevelSelection::iid(level.iid.clone());
        }

        // Check if the player is in bounds of another level
        for player_transform in &player_query {
            if player_transform.translation.x < level_bounds.max.x
                && player_transform.translation.x > level_bounds.min.x
                && player_transform.translation.y < level_bounds.max.y
                && player_transform.translation.y > level_bounds.min.y
                && !level_selection.is_match(&LevelIndices::default(), level)
            {
                *level_selection = LevelSelection::iid(level.iid.clone());
            }
        }
    }
}

/// Respawns the entire world for the currently selected ldtk gamefile
fn respawn_world(
    mut commands: Commands,
    ldtk_projects: Query<Entity, With<LdtkProjectHandle>>,
    input: Res<ButtonInput<KeyCode>>,
    game_file: Res<GameFile>,
    asset_server: Res<AssetServer>,
) {
    if input.just_pressed(KeyCode::KeyG) {
        commands.entity(ldtk_projects.single()).despawn_recursive();

        let ldtk_handle = asset_server
            .load(AssetPath::from_path(&game_file.path))
            .into();
        commands.spawn(LdtkWorldBundle {
            ldtk_handle,
            ..Default::default()
        });
    }
}

/// Respawn the current level and move the player to that level's respawn point.
/// If the level has not respawn point, do nothing
fn respawn_level(
    mut commands: Commands,
    ldtk_projects: Query<&LdtkProjectHandle>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
    input: Res<ButtonInput<KeyCode>>,

    level_selection: Res<LevelSelection>,
    levels: Query<(Entity, &LevelIid)>,
    player_respawns: Query<(&PlayerRespawn, &Transform), Without<Player>>,
    mut players: Query<&mut Transform, (With<Player>, Without<PlayerRespawn>)>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        // First, we have to find the level entity for the selected level
        if let LevelSelection::Iid(level_selection_iid) = level_selection.as_ref() {
            for (level_ent, level_iid) in levels.iter() {
                if level_iid == level_selection_iid {
                    // Then, get some level metadata
                    let ldtk_project = ldtk_project_assets
                        .get(ldtk_projects.single())
                        .expect("Project should be loaded during respawning");
                    let level = ldtk_project
                        .get_raw_level_by_iid(level_iid.get())
                        .expect("Level should be loaded during respawning");

                    // Find the correct respawn point for the currently level
                    // selected level
                    let opt_respawn_point = player_respawns.iter().find_map(
                        |(player_respawn, player_respawn_transform)| {
                            if player_respawn.level_uid == level.uid {
                                Some(Vec2::new(
                                    player_respawn_transform.translation.x,
                                    player_respawn_transform.translation.y,
                                ))
                            } else {
                                None
                            }
                        },
                    );

                    // Then, respawn the level and move the player to the
                    // respawn point
                    if let Some(respawn_point) = opt_respawn_point {
                        commands.entity(level_ent).insert(Respawn);
                        for mut player_transform in players.iter_mut() {
                            player_transform.translation.x = respawn_point.x;
                            player_transform.translation.y = respawn_point.y;
                        }
                    }
                }
            }
        }
    }
}

// Player respawn

/// tag for invisible location where the player should respawn
#[derive(Clone, Default, Debug, Component)]
pub struct PlayerRespawn {
    level_uid: i32,
}

/// The bundle of player respawn MUST be wordly. When this bundled is added,
/// we use the translation position provided by the auto-added `Transform`
/// component  to teleport the player to the respawn point. Becuaes the player
/// is wordly, this component must be, because level-based and world-based
/// components have different coordinate systems.
#[derive(Clone, Default, Debug, Bundle)]
pub struct PlayerRespawnBundle {
    player_respawn: PlayerRespawn,
    worldly: Worldly,
}

impl LdtkEntity for PlayerRespawnBundle {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        layer_instance: &LayerInstance,
        _: Option<&Handle<Image>>,
        _: Option<&TilesetDefinition>,
        _: &AssetServer,
        _: &mut Assets<TextureAtlasLayout>,
    ) -> Self {
        Self {
            player_respawn: PlayerRespawn {
                level_uid: layer_instance.level_id,
            },
            worldly: Worldly::from_entity_info(entity_instance),
        }
    }
}

// Plugin Wireup

pub struct GameFlowPlugin;

impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .register_ldtk_entity::<PlayerRespawnBundle>("Player_Respawn")
            .add_systems(Update, update_level_selection)
            .add_systems(Update, (respawn_world, respawn_level));
    }
}
