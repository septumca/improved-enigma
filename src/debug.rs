use std::{time::Duration};

use bevy::{prelude::*};

use crate::{
    GameState,
    yeti::{
        YetiSpawner,
        YetiSpawnPhase
    }
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
                ).in_set(OnUpdate(GameState::Playing)));
    }
}


fn spawn_debug_yeti(
    debug_controls: Res<DebugControls>,
    mut yeti_spawner: ResMut<YetiSpawner>,
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
        info!("YETI CONTROL {}", debug_controls.yeti_control);
    }
}
