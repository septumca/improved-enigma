use bevy::{prelude::*};

#[derive(Component)]
pub struct DebugMarker;

#[derive(Resource)]
struct DebugControls {
    is_debug_visible: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(DebugControls {
                is_debug_visible: false,
            })
            .add_system(debug_input);
    }
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
}