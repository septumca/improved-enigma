use std::{time::Duration, f32::consts::FRAC_PI_2};

use bevy::{prelude::*, math::vec2};
use rand::Rng;

use crate::{despawn, GameState, player::{PLAYER_Z_INDEX, Speed, Player}, Alive, SCALE_FACTOR, GameResources, collidable::Collidable, debug::DebugMarker, animation::Animation, trail::Trail};

const SPEED: f32 = 45.0 * SCALE_FACTOR;
const YETI_COLLIDABLE_DIMENSIONS: (f32, f32) = (3.0 * SCALE_FACTOR, 3.0 * SCALE_FACTOR);
const YETI_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -2.0 * SCALE_FACTOR);
const YETI_STUN_TIME: f32 = 1.5;
const YETI_SPAWNER_TIMEOUT: f32 = 30.0;


enum YetiSpawnPhase {
    Idle,
    Step,
    Spawned
}

#[derive(Resource)]
pub struct YetiSpawner {
    phase: YetiSpawnPhase,
    timer: Timer,
}

#[derive(Component, Debug, Eq, PartialEq)]
pub enum YetiState {
    Chasing,
    Stuned,
}

#[derive(Component)]
pub struct Yeti {
    pub stun_timer: Timer,
    pub ignore_collisions: Timer,
}

pub struct YetiPlugin;

impl Plugin for YetiPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(YetiSpawner {
                phase: YetiSpawnPhase::Idle,
                timer: Timer::from_seconds(YETI_SPAWNER_TIMEOUT, TimerMode::Once)
            })
            .add_systems(
                (
                    update_yeti_state.after(update_yeti),
                    update_spawner,
                    update_yeti
                ).in_set(OnUpdate(GameState::Playing)))
            .add_systems(
                (
                    despawn::<Yeti>,
                    reset_spawner
                ).in_schedule(OnExit(GameState::GameOver)));
    }
}

fn update_yeti_state(
    timer: Res<Time>,
    game_resources: Res<GameResources>,
    mut yeti_q: Query<(&mut Yeti, &mut YetiState, &mut Animation)>,
) {
    let Ok((mut yeti, mut yeti_state, mut animation)) = yeti_q.get_single_mut() else {
        return;
    };
    let dt = timer.delta();
    yeti.ignore_collisions.tick(dt);

    let new_state = match *yeti_state {
        YetiState::Chasing => None,
        YetiState::Stuned => {
            if yeti.stun_timer.tick(dt).finished() {
                yeti.ignore_collisions.reset();
                animation.set_frames(vec![
                    game_resources.yeti_run[0],
                    game_resources.yeti_run[1],
                    game_resources.yeti_run[0],
                    game_resources.yeti_run[2]
                ]);
                Some(YetiState::Chasing)
            } else {
                None
            }
        }
    };

    if let Some(new_state) = new_state {
        *yeti_state = new_state;
    }
}

fn update_yeti(
    timer: Res<Time>,
    mut yeti_q: Query<(&mut Transform, &YetiState, &Speed), Without<Player>>,
    player_q: Query<&Transform, (With<Player>, Without<Yeti>)>,
) {
    let Ok((
        mut yeti_transform,
        yeti_state,
        speed,
    )) = yeti_q.get_single_mut() else {
        return;
    };
    if yeti_state == &YetiState::Stuned {
        return;
    }
    let Ok(
        player_transform,
    ) = player_q.get_single() else {
        return;
    };
    let dt = timer.delta();
    let vel = (player_transform.translation.truncate() - yeti_transform.translation.truncate()).normalize() * speed.max_speed;
    yeti_transform.translation.x += vel.x * dt.as_secs_f32();
    yeti_transform.translation.y += vel.y * dt.as_secs_f32();
}

fn update_spawner(
    timer: Res<Time>,
    mut commands: Commands,
    mut yeti_spawner: ResMut<YetiSpawner>,
    window: Query<&Window>,
    game_resources: Res<GameResources>,
    camera_q: Query<&Transform, With<Camera>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let Ok(camera_transform) = camera_q.get_single() else {
        return;
    };
    if !yeti_spawner.timer.tick(timer.delta()).finished() {
        return;
    }
    let mut rng = rand::thread_rng();
    let next_phase = match yeti_spawner.phase {
        YetiSpawnPhase::Idle => {
            let mut x = -window.width() * 2.0;
            let y = camera_transform.translation.y - window.height();
            while x < window.width() * 2.0 {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                            rect: Some(game_resources.yeti_step),
                            ..default()
                        },
                        texture: game_resources.image_handle.clone(),
                        transform: Transform::from_xyz(x, y, PLAYER_Z_INDEX - 1.5).with_rotation(Quat::from_rotation_z(-FRAC_PI_2)),
                        ..default()
                    },
                    Trail
                ));
                x += game_resources.sprite_size;
            }
            YetiSpawnPhase::Step
        },
        YetiSpawnPhase::Step => {
            let mut stun_timer = Timer::from_seconds(YETI_STUN_TIME, TimerMode::Once);
            stun_timer.set_elapsed(Duration::from_secs_f32(YETI_STUN_TIME));
            let x = rng.gen_range(0.0..window.width());
            let y = camera_transform.translation.y + window.height();

            commands.spawn((
                SpriteBundle {
                  sprite: Sprite {
                      custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                      rect: Some(game_resources.yeti_run[0]),
                      ..default()
                  },
                  texture: game_resources.image_handle.clone(),
                  transform: Transform::from_xyz(x, y, PLAYER_Z_INDEX + 0.5),
                  ..default()
                },
                Collidable::new(
                    0., 0.,
                    YETI_COLLIDABLE_DIMENSIONS.0, YETI_COLLIDABLE_DIMENSIONS.1,
                    YETI_COLLIDABLE_OFFSETS.0, YETI_COLLIDABLE_OFFSETS.1
                ),
                Speed::new(SPEED, 1.0),
                Alive,
                Animation::new(vec![
                    game_resources.yeti_run[0],
                    game_resources.yeti_run[1],
                    game_resources.yeti_run[0],
                    game_resources.yeti_run[2]
                ], TimerMode::Repeating),
                Yeti {
                    stun_timer,
                    ignore_collisions: Timer::from_seconds(0.5, TimerMode::Once)
                },
                YetiState::Chasing,
          ))
          .with_children(|parent| {
              parent.spawn((
                  SpriteBundle {
                      sprite: Sprite {
                          color: Color::rgb(0.25, 0.25, 0.75),
                          custom_size: Some(Vec2::new(YETI_COLLIDABLE_DIMENSIONS.0 * 2.0, YETI_COLLIDABLE_DIMENSIONS.1 * 2.0)),
                          ..default()
                      },
                      transform: Transform::from_xyz(YETI_COLLIDABLE_OFFSETS.0, YETI_COLLIDABLE_OFFSETS.1, 1.0),
                      visibility: Visibility::Hidden,
                      ..default()
                  },
                  DebugMarker
              ));
          });
          yeti_spawner.timer.set_duration(Duration::from_secs_f32(2.0));
          YetiSpawnPhase::Spawned
        },
        YetiSpawnPhase::Spawned => {
            yeti_spawner.timer.paused();
            YetiSpawnPhase::Spawned
        }
    };

    yeti_spawner.phase = next_phase;
    yeti_spawner.timer.reset();
}

fn reset_spawner(
    mut yeti_spawner: ResMut<YetiSpawner>,
) {
    yeti_spawner.phase = YetiSpawnPhase::Idle;
    yeti_spawner.timer = Timer::from_seconds(YETI_SPAWNER_TIMEOUT, TimerMode::Once);
}