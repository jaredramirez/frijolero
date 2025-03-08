use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::player::Player;

const ASPECT_RATIO_HEIGHT: f32 = 9.;
const ASPECT_RATIO_WIDTH: f32 = 16.;
const ASPECT_RATIO: f32 = ASPECT_RATIO_WIDTH / ASPECT_RATIO_HEIGHT;

const CAMERA_HEIGHT: f32 = 200.0;
const CAMERA_WIDTH: f32 = CAMERA_HEIGHT * ASPECT_RATIO;

const CAMERA_HEIGHT_HALF: f32 = CAMERA_HEIGHT / 2.;
const CAMERA_WIDTH_HALF: f32 = CAMERA_WIDTH / 2.;

#[allow(clippy::type_complexity)]
pub fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (
            &mut bevy::render::camera::OrthographicProjection,
            &mut Transform,
        ),
        Without<Player>,
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<(&Transform, &LevelIid), (Without<OrthographicProjection>, Without<Player>)>,
    ldtk_projects: Query<&LdtkProjectHandle>,
    level_selection: Res<LevelSelection>,
    ldtk_project_assets: Res<Assets<LdtkProject>>,
) {
    if let Ok(Transform {
        translation: player_translation,
        ..
    }) = player_query.get_single()
    {
        let player_translation = *player_translation;

        let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();

        for (level_transform, level_iid) in &level_query {
            let ldtk_project = ldtk_project_assets
                .get(ldtk_projects.single())
                .expect("Project should be loaded if level has spawned");

            let level = ldtk_project
                .get_raw_level_by_iid(&level_iid.to_string())
                .expect("Spawned level should exist in LDtk project");

            if level_selection.is_match(&LevelIndices::default(), level) {
                let level_width = level.px_wid as f32;
                let level_height = level.px_hei as f32;
                orthographic_projection.viewport_origin = Vec2::ZERO;

                if level_width < CAMERA_WIDTH {
                    let width = (level_width / ASPECT_RATIO_WIDTH).round() * ASPECT_RATIO_WIDTH;
                    let height = width / ASPECT_RATIO;
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed { width, height };

                    // Convert the wordly player coords into level coords,
                    // fit the camera to the level, then convert back to world
                    // coords
                    camera_transform.translation.y =
                        (player_translation.y - level_transform.translation.y - height / 2.)
                            .clamp(0., level_height - height)
                            + level_transform.translation.y;
                    camera_transform.translation.x = level_transform.translation.x;
                } else if level_height < CAMERA_HEIGHT {
                    let height = (level_height / ASPECT_RATIO_HEIGHT).round() * ASPECT_RATIO_HEIGHT;
                    let width = height * ASPECT_RATIO;
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed { width, height };

                    // Convert the wordly player coords into level coords,
                    // fit the camera to the level, then convert back to world
                    // coords
                    camera_transform.translation.x =
                        (player_translation.x - level_transform.translation.x - width / 2.)
                            .clamp(0., level_width - width)
                            + level_transform.translation.x;

                    camera_transform.translation.y = level_transform.translation.y;
                } else {
                    orthographic_projection.scaling_mode =
                        bevy::render::camera::ScalingMode::Fixed {
                            width: CAMERA_WIDTH,
                            height: CAMERA_HEIGHT,
                        };

                    // Convert the wordly player coords  into level
                    let player_x_in_level = player_translation.x - level_transform.translation.x;
                    let player_y_in_level = player_translation.y - level_transform.translation.y;

                    // Make the camera follow the player, fitting camera inside level
                    let camera_x_in_level =
                        player_x_in_level.clamp(CAMERA_WIDTH_HALF, level_width - CAMERA_WIDTH_HALF);
                    let camera_y_in_level = player_y_in_level
                        .clamp(CAMERA_HEIGHT_HALF, level_height - CAMERA_HEIGHT_HALF);

                    // Center the camera on the player, & convert level coords back
                    // into world coords
                    camera_transform.translation.x =
                        camera_x_in_level - CAMERA_WIDTH_HALF + level_transform.translation.x;
                    camera_transform.translation.y =
                        camera_y_in_level - CAMERA_HEIGHT_HALF + level_transform.translation.y;
                }

                // if level_ratio > ASPECT_RATIO {
                //     // level is wider than the screen
                //     let height = (level.px_hei as f32 / 9.).round() * 9.;
                //     let width = height * ASPECT_RATIO;
                //     orthographic_projection.scaling_mode =
                //         bevy::render::camera::ScalingMode::Fixed { width, height };
                //     camera_transform.translation.x =
                //         (player_translation.x - level_transform.translation.x - width / 2.)
                //             .clamp(0., level_width - width);
                //     camera_transform.translation.y = 0.;
                // } else {
                //     // level is taller than the screen
                //     let width = (level.px_wid as f32 / 16.).round() * 16.;
                //     let height = width / ASPECT_RATIO;
                //     orthographic_projection.scaling_mode =
                //         bevy::render::camera::ScalingMode::Fixed { width, height };
                //     camera_transform.translation.y =
                //         (player_translation.y - level_transform.translation.y - height / 2.)
                //             .clamp(0., level.px_hei as f32 - height);
                //     camera_transform.translation.x = 0.;
                // }
                //
                // camera_transform.translation.x += level_transform.translation.x;
                // camera_transform.translation.y += level_transform.translation.y;
            }
        }
    }
}
