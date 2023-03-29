use std::time::Duration;

use bevy::{prelude::*};

use crate::{
    GameState,
    yeti::{
        Yeti,
        update_yeti,
        YetiSpawner,
        YetiSpawnPhase, YetiAi
    },
};

#[derive(Component)]
pub struct DebugMarker;

#[derive(Resource)]
struct DebugControls {
    is_debug_visible: bool,
    yeti_control: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(DebugControls {
                is_debug_visible: false,
                yeti_control: false
            })
            .add_system(debug_input)
            .add_systems(
                (
                    spawn_debug_yeti,
                    mouse_input.before(update_yeti),
                    update_camera.after(update_yeti),
                ).in_set(OnUpdate(GameState::Playing)));
    }
}

fn mouse_input(
    mouse_button_input: Res<Input<MouseButton>>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    mut yeti_q: Query<&mut Yeti, Without<Camera>>
) {
    let Ok(mut yeti) = yeti_q.get_single_mut() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };

    if mouse_button_input.pressed(MouseButton::Left) {
        let Some(mouse_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate()) else
        {
            return;
        };

        yeti.target_position = Some(mouse_position);
    }
}

fn spawn_debug_yeti(
    mut commands: Commands,
    debug_controls: Res<DebugControls>,
    mut yeti_spawner: ResMut<YetiSpawner>,
    yeti_q: Query<Entity, (With<Yeti>, With<YetiAi>)>,
) {
    if !debug_controls.yeti_control {
        return;
    }
    if let Some(phase_override) = match yeti_spawner.phase {
        YetiSpawnPhase::Idle | YetiSpawnPhase::Step => Some(YetiSpawnPhase::Spawning),
        _ => None
    } {
        info!("forwarding phase to spawning phase");
        yeti_spawner.phase = phase_override;
        yeti_spawner.timer.set_elapsed(Duration::from_secs_f32(1000.));
    };
    if let Ok(entity) = yeti_q.get_single() {
        info!("removing yeti ai");
        commands.entity(entity).remove::<YetiAi>();
    }
}

fn update_camera(
    debug_controls: Res<DebugControls>,
    mut yeti_q: Query<(&Transform, &mut Yeti), Without<Camera>>,
    mut camera_q: Query<&mut Transform, With<Camera>>
) {
    if !debug_controls.yeti_control {
        return;
    }
    let Ok((yeti_transform, mut yeti)) = yeti_q.get_single_mut() else {
        return;
    };
    if yeti.target_position.map_or(false, |p| p.distance_squared(yeti_transform.translation.truncate()) < 10.0) {
        yeti.target_position = None;
    }
    let Ok(mut camera_transform) = camera_q.get_single_mut() else {
        return;
    };
    camera_transform.translation.x = yeti_transform.translation.x;
    camera_transform.translation.y = yeti_transform.translation.y;
}

fn debug_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut debug_controls: ResMut<DebugControls>,
    mut debug_q: Query<&mut Visibility, With<DebugMarker>>,
) {
    if keyboard_input.just_pressed(KeyCode::O) {
        debug_controls.is_debug_visible = !debug_controls.is_debug_visible;

        for mut visibility in debug_q.iter_mut() {
            match debug_controls.is_debug_visible {
                true => *visibility = Visibility::Inherited,
                false => *visibility = Visibility::Hidden,
            };
        }
    }

    if keyboard_input.just_pressed(KeyCode::I) {
        debug_controls.yeti_control = !debug_controls.yeti_control;
    }
}
