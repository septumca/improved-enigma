use bevy::{prelude::*};

use crate::{player::{update_movables, PLAYER_CAMERA_OFFSET}, GameState};

#[derive(Component)]
pub struct CameraFocus;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                (
                  update_camera.after(update_movables),
                ).in_set(OnUpdate(GameState::Playing)));
    }
}

fn update_camera(
  focus_q: Query<&Transform, (With<CameraFocus>, Without<Camera>)>,
  mut camera_q: Query<&mut Transform, With<Camera>>,
) {
  let Ok(focus_transform) = focus_q.get_single() else {
      return;
  };
  let Ok(mut camera_transform) = camera_q.get_single_mut() else {
      return;
  };
  camera_transform.translation.x = focus_transform.translation.x;
  camera_transform.translation.y = focus_transform.translation.y - PLAYER_CAMERA_OFFSET;
}