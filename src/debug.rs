use std::{time::Duration, f32::consts::PI};

use bevy::{prelude::*, math::vec2};

use crate::{
    GameState,
    yeti::{
        YetiSpawner,
        YetiSpawnPhase, Yeti
    }, player::Player, GameResources, animation::AnimateRotation, SCALE_FACTOR, stuneffect::{StunEffect, Stun}
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
    mut commands: Commands,
    game_resources: Res<GameResources>,
    keyboard_input: Res<Input<KeyCode>>,
    mut debug_controls: ResMut<DebugControls>,
    mut debug_q: Query<&mut Visibility, With<DebugMarker>>,
    stunnable_q: Query<Entity, Or<(With<Yeti>, With<Player>)>>
) {
    if keyboard_input.just_pressed(KeyCode::O) {
        debug_controls.is_debug_visible = !debug_controls.is_debug_visible;
        info!("TOGGLE DEBUG {}", debug_controls.is_debug_visible);

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

    if keyboard_input.just_pressed(KeyCode::U) {
        info!("ADDING STUN");
        for entity in stunnable_q.iter() {
            let stun_child = commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                        rect: Some(game_resources.stun),
                        ..default()
                    },
                    texture: game_resources.image_handle.clone(),
                    transform: Transform::from_xyz(0.0, 4.0 * SCALE_FACTOR, 0.5),
                    ..default()
                },
                AnimateRotation {
                    angular_vel: PI
                },
                StunEffect
            )).id();
            commands.entity(entity).push_children(&[stun_child]);
            commands.entity(entity).insert(Stun(Timer::from_seconds(0.5, TimerMode::Once)));
        }
    }

}
