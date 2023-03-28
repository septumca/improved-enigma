use bevy::{prelude::*, math::vec2};

use crate::{despawn, GameState, player::{Player, Direction, PLAYER_Z_INDEX}, Alive, SCALE_FACTOR, cleanup, GameResources, yeti::{Yeti}, animation::Animation, stuneffect::Stun};


const TRAIL_SIZE: (f32, f32) = (1.0 * SCALE_FACTOR, 1.0 * SCALE_FACTOR);

#[derive(Component)]
pub struct Trail;

pub struct TrailPlugin;

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                (
                    leave_trail_yeti,
                    leave_trail_player,
                    cleanup::<Trail>,
                ).in_set(OnUpdate(GameState::Playing)))
            .add_system(despawn::<Trail>.in_schedule(OnExit(GameState::GameOver)));
    }
}

fn leave_trail_yeti(
    mut commands: Commands,
    game_resources: Res<GameResources>,
    yeti_q: Query<(&Transform, &Animation), (Changed<Transform>, With<Yeti>, With<Alive>, Without<Stun>)>,
) {
    let Ok((transform, animation)) = yeti_q.get_single() else {
        return;
    };

    if animation.timer.just_finished() &&
        (animation.act_frame_index == 0 || animation.act_frame_index == 2)
    {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                    rect: Some(game_resources.yeti_step),
                    ..default()
                },
                texture: game_resources.image_handle.clone(),
                transform: Transform::from_xyz(transform.translation.x, transform.translation.y, PLAYER_Z_INDEX - 1.5),
                ..default()
            },
            Trail
        ));
    }
}

fn leave_trail_player(
    mut commands: Commands,
    player_q: Query<(&Transform, &Direction), (Changed<Transform>, With<Player>, With<Alive>)>,
) {
    let Ok((transform, direction)) = player_q.get_single() else {
        return;
    };

    let offsets = direction.get_standing_position_offset();

    for (dx, dy) in offsets {
        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.8, 0.8, 0.9),
                    custom_size: Some(Vec2::new(TRAIL_SIZE.0, TRAIL_SIZE.1)),
                    ..default()
                },
                transform: Transform::from_xyz(transform.translation.x + dx, transform.translation.y + dy, 0.),
                ..default()
            },
            Trail
        ));
    }
}