use std::{time::Duration};
use bevy::{prelude::*, math::vec2};
use rand::{self, Rng, seq::SliceRandom};

use crate::{collidable::Collidable, Alive, GameState, SCREEN_HEIGHT, SCALE_FACTOR, GameResources, SCREEN_WIDTH, SPRITE_SIZE, player::{Player, Slowdown, FALL_TIMEOUT}, cleanup};

const TREE_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.5, 3.5);
const STONE_COLLIDABLE_DIMENSIONS: (f32, f32) = (3.0, 3.0);
const OBSTACLE_SPAWN_TIME: (f32, f32) = (0.1, 0.6);

#[derive(Resource)]
struct ObstacleSpawner {
    timer: Timer,
    last_spawned_y: f32,
}

#[derive(Component)]
struct Obstacle;

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ObstacleSpawner {
                timer: Timer::from_seconds(OBSTACLE_SPAWN_TIME.0, TimerMode::Repeating),
                last_spawned_y: -SCREEN_HEIGHT / SCALE_FACTOR as f32 * 0.6
            })
            .add_systems(
                (
                    spawn_obstales,
                    update_collidables,
                    process_collisions.after(update_collidables),
                    cleanup::<Obstacle>,
                ).in_set(OnUpdate(GameState::Playing))
            );
    }
}


fn spawn_obstales(
    mut commands: Commands,
    timer: Res<Time>,
    sprite_rects: Res<GameResources>,
    camera_q: Query<&Transform, With<Camera>>,
    mut spawner_r: ResMut<ObstacleSpawner>,
) {
    let Ok(transform) = camera_q.get_single() else {
        return;
    };
    let Some(texture_handle) = &sprite_rects.image_handle else {
        return;
    };
    if (spawner_r.last_spawned_y - transform.translation.y).abs() > SCREEN_HEIGHT {
        return;
    };
    let mut rng = rand::thread_rng();

    if spawner_r.timer.tick(timer.delta()).just_finished() {
        let offset_y = (transform.translation.y - SCREEN_HEIGHT / SCALE_FACTOR as f32 * 0.6).min(spawner_r.last_spawned_y);
        let dimensions_x = (SCREEN_WIDTH - SPRITE_SIZE) / SCALE_FACTOR as f32 * 0.5;
        let x = rng.gen_range(-dimensions_x..dimensions_x) + transform.translation.x;
        let y = offset_y - rng.gen_range(4.0..8.0);
        let next_timeout = rng.gen_range(OBSTACLE_SPAWN_TIME.0..OBSTACLE_SPAWN_TIME.1);
        let choices = [
            (sprite_rects.tree, TREE_COLLIDABLE_DIMENSIONS),
            (sprite_rects.stone, STONE_COLLIDABLE_DIMENSIONS)
        ];
        let Some((sprite_rect, collidable_dimension)) = choices.choose(&mut rng) else {
            return;
        };

        spawner_r.timer.set_duration(Duration::from_secs_f32(next_timeout));
        spawner_r.timer.reset();
        spawner_r.last_spawned_y = y;

        commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(vec2(sprite_rects.sprite_size, sprite_rects.sprite_size)),
                    rect: Some(sprite_rect.clone()),
                    ..default()
                },
                texture: texture_handle.clone(),
                transform: Transform::from_xyz(x, y, 0.),
                ..default()
            },
            Collidable::new(x, y, collidable_dimension.0, collidable_dimension.1),
            Alive {},
            Obstacle {}
        ))
        // .with_children(|parent| {
        //     parent.spawn(SpriteBundle {
        //         sprite: Sprite {
        //             color: Color::rgb(0.75, 0.25, 0.25),
        //             custom_size: Some(Vec2::new(collidable_dimension.0 * 2.0, collidable_dimension.1 * 2.0)),
        //             ..default()
        //         },
        //         ..default()
        //     });
        // })
        ;
    }
}


fn update_collidables(
    mut collidables_q: Query<(&Transform, &mut Collidable)>
) {
    for (transform, mut collidable) in collidables_q.iter_mut() {
        collidable.update_center(transform.translation.x, transform.translation.y);
    }
}

fn process_collisions(
    mut commands: Commands,
    sprite_rects: Res<GameResources>,
    mut player_q: Query<(Entity, &mut Sprite, &Collidable), (With<Player>, With<Alive>, Without<Obstacle>)>,
    mut obstacles_q: Query<&Collidable, (With<Obstacle>, Without<Player>)>
) {
    let Ok((entity, mut sprite, collidable_player)) = player_q.get_single_mut() else {
        return;
    };

    let has_collided = obstacles_q.iter_mut().any(|collidable_obstacle| {
        collidable_obstacle.intersect(&collidable_player)
    });

    if has_collided {
        commands.entity(entity).remove::<Alive>();
        commands.entity(entity).insert(Slowdown(Timer::from_seconds(FALL_TIMEOUT, TimerMode::Once)));
        sprite.rect = Some(sprite_rects.fall_down);
    }
}
