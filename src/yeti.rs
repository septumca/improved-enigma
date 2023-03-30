use std::{time::Duration, f32::consts::{FRAC_PI_2, FRAC_PI_8}};

use bevy::{prelude::*, math::vec2};
use rand::Rng;

use crate::{despawn, GameState, player::{PLAYER_Z_INDEX, Player, PLAYER_CAMERA_OFFSET, Velocity}, Alive, SCALE_FACTOR, GameResources, collidable::{Collidable, CollidableMovable}, debug::DebugMarker, animation::Animation, trail::Trail, stuneffect::{Stun, self}, obstacle::Obstacle};

const SPEED: f32 = 48.0 * SCALE_FACTOR;
const YETI_COLLIDABLE_DIMENSIONS: (f32, f32) = (4.0 * SCALE_FACTOR, 3.0 * SCALE_FACTOR);
const YETI_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -2.0 * SCALE_FACTOR);
pub const YETI_STUN_TIME: f32 = 0.8;
const YETI_SPAWNER_TIMEOUT: (f32, f32) = (30.0, 3.0);
const RAY_LENGHT: f32 = 400.0;

#[derive(Component)]
pub enum YetiAi {
    ChaseDirect,
    ChaseAvoidObstacles,
}

pub enum YetiSpawnPhase {
    Idle,
    Step,
    Spawning,
    Completed
}

#[derive(Resource)]
pub struct YetiSpawner {
    pub phase: YetiSpawnPhase,
    pub timer: Timer,
}

#[derive(Component)]
pub struct Yeti {
    pub ignore_collisions: Timer,
    pub speed: f32
}

pub struct YetiPlugin;

#[derive(Component)]
struct DebugYetiVelocity;

impl Plugin for YetiPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(YetiSpawner {
                phase: YetiSpawnPhase::Idle,
                timer: Timer::from_seconds(YETI_SPAWNER_TIMEOUT.0, TimerMode::Once)
            })
            .add_systems(
                (
                    update_spawner,
                    yeti_ai,
                    update_yeti.after(yeti_ai),
                    update_wake_up.after(stuneffect::update_stun),
                ).in_set(OnUpdate(GameState::Playing)))
            .add_systems(
                (
                    despawn::<Yeti>,
                    reset_spawner
                ).in_schedule(OnEnter(GameState::Playing)));
    }
}

pub fn update_wake_up(
    game_resources: Res<GameResources>,
    mut woken_up: RemovedComponents<Stun>,
    mut yeti_q: Query<(&mut Sprite, &mut Animation, &mut Yeti), With<Alive>>,
) {
    for entity_woken in woken_up.iter() {
        let Ok((mut sprite, mut animation, mut yeti)) = yeti_q.get_mut(entity_woken) else {
            continue;
        };

        yeti.ignore_collisions.reset();
        animation.set_frames(vec![
            game_resources.yeti_run[0],
            game_resources.yeti_run[1],
            game_resources.yeti_run[0],
            game_resources.yeti_run[2]
        ]);
        if let Some(rect) = animation.get_frame() {
            sprite.rect = Some(rect);
        }
    }
}


fn yeti_ai(
    obstacles_q: Query<&Collidable, (With<Obstacle>, Without<Yeti>)>,
    mut yeti_q: Query<(&Transform, &Yeti, &mut Velocity, &YetiAi), (Without<Player>, Without<Stun>)>,
    player_q: Query<&Transform, (With<Player>, Without<Yeti>)>,
) {
    let Ok((
        transform,
        yeti,
        mut velocity,
        yeti_ai,
    )) = yeti_q.get_single_mut() else {
        return;
    };
    let Ok(
        player_transform,
    ) = player_q.get_single() else {
        velocity.0 = Vec2::ZERO;
        return;
    };

    match yeti_ai {
        YetiAi::ChaseDirect => {
            velocity.0 = (player_transform.translation.truncate() - transform.translation.truncate()).normalize() * yeti.speed;
        },
        YetiAi::ChaseAvoidObstacles => {
            let position = transform.translation.truncate();
            let facing_vec = player_transform.translation.truncate() - position;
            let rotation = facing_vec.y.atan2(facing_vec.x);
            let ray_length = facing_vec.length().min(RAY_LENGHT);

            for i in 0..16 {
                let angle_offset = rotation + (FRAC_PI_8 / 2.0 * i as f32);
                let angle_offset_neg = rotation - (FRAC_PI_8 / 2.0 * i as f32);
                let ray = vec2(angle_offset.cos(), angle_offset.sin());
                let ray_neg = vec2(angle_offset_neg.cos(), angle_offset_neg.sin());

                let mut intersect = false;
                let mut intersect_neg = false;
                for collidable_obstacle in obstacles_q.iter() {
                    if !intersect {
                        if collidable_obstacle.intersect_line(
                            position,
                            position + ray * ray_length
                        ).is_some() {
                            intersect = true;
                        };
                    }
                    if !intersect_neg {
                        if collidable_obstacle.intersect_line(
                            position,
                            position + ray_neg * ray_length
                        ).is_some() {
                            intersect_neg = true;
                        };
                    }
                    if intersect && intersect_neg {
                        break;
                    }
                };
                if !intersect {
                    if i == 0 {
                        println!("no correction necessary at angle {}", angle_offset);
                    } else {
                        println!("found angle {} at offset {}", angle_offset, i);
                    }
                    velocity.0 = ray * yeti.speed;
                    return;
                }
                if !intersect_neg {
                    println!("found negative angle {} at offset {}", angle_offset_neg, -i);
                    velocity.0 = ray_neg * yeti.speed;
                    return;
                }
            }
        }
    };
}

pub fn update_yeti(
    timer: Res<Time>,
    mut yeti_q: Query<&mut Yeti, Without<Stun>>,
) {
    let Ok(mut yeti) = yeti_q.get_single_mut() else {
        return;
    };
    let dt = timer.delta();
    yeti.ignore_collisions.tick(dt);
}

pub fn update_spawner(
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
            YetiSpawnPhase::Step
        },
        YetiSpawnPhase::Step => {
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
            yeti_spawner.timer.set_duration(Duration::from_secs_f32(YETI_SPAWNER_TIMEOUT.1));
            YetiSpawnPhase::Spawning
        },
        YetiSpawnPhase::Spawning => {
            let x = camera_transform.translation.x + window.width() / 2.0;
            let y = camera_transform.translation.y + rng.gen_range(0.0..(window.height() / 2.0 - PLAYER_CAMERA_OFFSET));

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
                CollidableMovable,
                Collidable::new(
                    0., 0.,
                    YETI_COLLIDABLE_DIMENSIONS.0, YETI_COLLIDABLE_DIMENSIONS.1,
                    YETI_COLLIDABLE_OFFSETS.0, YETI_COLLIDABLE_OFFSETS.1
                ),
                Alive,
                Velocity(Vec2::ZERO),
                Animation::new(vec![
                    game_resources.yeti_run[0],
                    game_resources.yeti_run[1],
                    game_resources.yeti_run[0],
                    game_resources.yeti_run[2]
                ], TimerMode::Repeating),
                Yeti {
                    ignore_collisions: Timer::from_seconds(1.5, TimerMode::Once),
                    speed: SPEED
                },
                YetiAi::ChaseDirect,
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
            yeti_spawner.timer.paused();
            YetiSpawnPhase::Completed
        },
        YetiSpawnPhase::Completed => YetiSpawnPhase::Completed
    };

    yeti_spawner.phase = next_phase;
    yeti_spawner.timer.reset();
}

fn reset_spawner(
    mut yeti_spawner: ResMut<YetiSpawner>,
) {
    yeti_spawner.phase = YetiSpawnPhase::Idle;
    yeti_spawner.timer = Timer::from_seconds(YETI_SPAWNER_TIMEOUT.0, TimerMode::Once);
}