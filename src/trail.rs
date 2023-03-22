use bevy::{prelude::*};

use crate::{despawn, GameState, player::{Player, Direction}, Alive, SCALE_FACTOR, cleanup};


const TRAIL_SIZE: (f32, f32) = (1.0 * SCALE_FACTOR, 1.0 * SCALE_FACTOR);

#[derive(Component)]
struct Trail;


pub struct TrailPlugin;

impl Plugin for TrailPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                (
                    leave_trail,
                    cleanup::<Trail>,
                ).in_set(OnUpdate(GameState::Playing)))
            .add_system(despawn::<Trail>.in_schedule(OnExit(GameState::GameOver)));
    }
}

fn leave_trail(
    mut commands: Commands,
    player_q: Query<(&Transform, &Direction), (With<Player>, With<Alive>)>,
) {
    let Ok((transform, direction)) = player_q.get_single() else {
        return;
    };

    let offsets = match direction {
        Direction::Down | Direction::Right(0) | Direction::Left(0) => vec![(-2.* SCALE_FACTOR, -1.* SCALE_FACTOR), (2.* SCALE_FACTOR, -1.* SCALE_FACTOR)],
        Direction::Right(1) | Direction::Left(1) => vec![(-2.* SCALE_FACTOR, -2.* SCALE_FACTOR), (2.* SCALE_FACTOR, -2.* SCALE_FACTOR)],
        Direction::Right(2) | Direction::Left(2) => vec![(-2.* SCALE_FACTOR, -3.* SCALE_FACTOR), (2.* SCALE_FACTOR, -3.* SCALE_FACTOR)],
        Direction::Right(3) => vec![(-2.* SCALE_FACTOR, -4.* SCALE_FACTOR), (2.* SCALE_FACTOR, -3.* SCALE_FACTOR)],
        Direction::Left(3) => vec![(-2.* SCALE_FACTOR, -3.* SCALE_FACTOR), (2.* SCALE_FACTOR, -4.* SCALE_FACTOR)],
        _ => vec![]
    };

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