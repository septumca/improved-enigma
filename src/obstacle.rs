use bevy::{prelude::*, math::vec2, utils::HashSet};
use rand::{self, Rng, seq::SliceRandom};

use crate::{collidable::Collidable, Alive, GameState, SCREEN_HEIGHT, SCALE_FACTOR, GameResources, player::{Player, Slowdown, FALL_TIMEOUT}, cleanup, despawn, SCREEN_WIDTH, debug::DebugMarker, SPRITE_SIZE};

const TREE_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
const TREE_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -2.0 * SCALE_FACTOR);
const STONE_COLLIDABLE_DIMENSIONS: (f32, f32) = (3.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
const STONE_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -1.0 * SCALE_FACTOR);
const REGION_OFFSETS: [(isize, isize); 5] = [(-1, 0), (-1, -1), (0, -1), (1, -1), (1, 0)];
const REGION_WIDTH: f32 = SCREEN_WIDTH;
const REGION_HEIGHT: f32 = SCREEN_HEIGHT;
const GAP_RANGE: (f32, f32) = (SPRITE_SIZE * SCALE_FACTOR * 1.0, SPRITE_SIZE * SCALE_FACTOR * 5.0);

#[derive(Clone)]
enum ObstacleType {
    Stone,
    Tree,
}

struct XYFillSpawner {
    current_filled_x: f32,
    current_filled_y: f32,
    y_gap: f32,
}

fn xy_walk_spawner() -> XYFillSpawner {
    let mut rng = rand::thread_rng();
    let y_gap = rng.gen_range(GAP_RANGE.0..GAP_RANGE.1);
    let current_filled_y = y_gap * 2.0;
    XYFillSpawner {
        current_filled_x: rng.gen_range(GAP_RANGE.0..GAP_RANGE.1),
        current_filled_y,
        y_gap,
    }
}

impl Iterator for XYFillSpawner {
    type Item = (ObstacleType, f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_filled_y > REGION_HEIGHT {
            return None;
        }
        let mut rng = rand::thread_rng();
        let choices = [ObstacleType::Tree, ObstacleType::Stone];
        let Some(obstacle_type) = choices.choose(&mut rng) else {
            return None;
        };

        let data = (obstacle_type.clone(), self.current_filled_x, self.current_filled_y + rng.gen_range(-self.y_gap * 0.75..self.y_gap * 0.75));
        if self.current_filled_x + GAP_RANGE.1 > REGION_WIDTH {
            self.current_filled_x = rng.gen_range(GAP_RANGE.0..GAP_RANGE.1);
            self.y_gap = rng.gen_range(GAP_RANGE.0..GAP_RANGE.1);
            self.current_filled_y = self.current_filled_y + self.y_gap;
        } else {
            self.current_filled_x = self.current_filled_x + rng.gen_range(GAP_RANGE.0..GAP_RANGE.1);
        }

        Some(data)
    }
}


#[derive(Resource)]
struct ObstacleSpawner {
    last_processed_player_region_y: Option<isize>,
    last_processed_player_region_x: Option<isize>,
    spawned_regions: HashSet<(isize, isize)>,
}

#[derive(Component)]
struct Obstacle;

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ObstacleSpawner {
                last_processed_player_region_y: None,
                last_processed_player_region_x: None,
                spawned_regions: HashSet::new(),
            })
            .add_system(despawn::<Obstacle>.in_schedule(OnExit(GameState::GameOver)))
            .add_system(reset_spawner.in_schedule(OnExit(GameState::GameOver)))
            .add_systems(
                (
                    spawn_obstales,
                    update_collidables,
                    process_collisions.after(update_collidables),
                    cleanup::<Obstacle>,
                    cleanup_regions.after(spawn_obstales),
                ).in_set(OnUpdate(GameState::Playing))
            );
    }
}


fn spawn_obstales(
    mut commands: Commands,
    game_resources: Res<GameResources>,
    camera_q: Query<&Transform, With<Camera>>,
    mut spawner_r: ResMut<ObstacleSpawner>,
) {
    let Ok(transform) = camera_q.get_single() else {
        return;
    };
    let Some(texture_handle) = &game_resources.image_handle else {
        return;
    };
    let camera_region_x = ((transform.translation.x + REGION_WIDTH / 2.0) / REGION_WIDTH).floor() as isize;
    let camera_region_y = (transform.translation.y / REGION_HEIGHT).floor() as isize;
    if spawner_r.last_processed_player_region_x.unwrap_or(camera_region_x+1) == camera_region_x &&
        spawner_r.last_processed_player_region_y.unwrap_or(camera_region_y+1) == camera_region_y  {
        return;
    }
    spawner_r.last_processed_player_region_x = Some(camera_region_x);
    spawner_r.last_processed_player_region_y = Some(camera_region_y);

    for (rox, roy) in REGION_OFFSETS {
        let rx = camera_region_x + rox;
        let ry = camera_region_y + roy;
        if spawner_r.spawned_regions.contains(&(rx, ry)) {
            continue;
        }

        for (obstacle_type, ox, oy) in xy_walk_spawner() {
            let x = rx as f32 * REGION_WIDTH + ox;
            let y = ry as f32 * REGION_HEIGHT - oy;

            let (sprite_rect, collidable_dimension, offsets) = match obstacle_type {
                ObstacleType::Tree => (game_resources.tree, TREE_COLLIDABLE_DIMENSIONS, TREE_COLLIDABLE_OFFSETS),
                ObstacleType::Stone => (game_resources.stone, STONE_COLLIDABLE_DIMENSIONS, STONE_COLLIDABLE_OFFSETS)
            };

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                        rect: Some(sprite_rect.clone()),
                        ..default()
                    },
                    texture: texture_handle.clone(),
                    transform: Transform::from_xyz(x, y, 0.),
                    ..default()
                },
                Collidable::new(x, y, collidable_dimension.0, collidable_dimension.1, offsets.0, offsets.1),
                Alive,
                Obstacle
            ))
            .with_children(|parent| {
                parent.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.75, 0.25, 0.25),
                            custom_size: Some(Vec2::new(collidable_dimension.0 * 2.0, collidable_dimension.1 * 2.0)),
                            ..default()
                        },
                        transform: Transform::from_xyz(offsets.0, offsets.1, 2.),
                        visibility: Visibility::Hidden,
                        ..default()
                    },
                    DebugMarker
                ));
            })
            ;
        }
        spawner_r.spawned_regions.insert((rx, ry));
    }
}

fn cleanup_regions(
    camera_q: Query<&Transform, With<Camera>>,
    mut spawner_r: ResMut<ObstacleSpawner>,
) {
    let Ok(transform) = camera_q.get_single() else {
        return;
    };

    let camera_region_y = (transform.translation.y / REGION_HEIGHT).floor() as isize;
    spawner_r.spawned_regions.retain(|(_, y)| y <= &camera_region_y);
}

fn reset_spawner(
    mut spawner_r: ResMut<ObstacleSpawner>,
) {
    spawner_r.last_processed_player_region_y = None;
    spawner_r.last_processed_player_region_x = None;
    spawner_r.spawned_regions = HashSet::new();
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
