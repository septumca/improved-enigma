use std::f32::consts::PI;

use bevy::{prelude::*, math::vec2, utils::HashSet};
use rand::{self, Rng, seq::SliceRandom};

use crate::{
    collidable::Collidable,
    Alive,
    GameState,
    SCALE_FACTOR,
    GameResources,
    player::{
        Player,
        Slowdown,
        FALL_TIMEOUT,
        PLAYER_Z_INDEX, LeftSki, RightSki, self, Catched
    },
    cleanup,
    despawn,
    debug::DebugMarker,
    SPRITE_SIZE, yeti::{Yeti, YETI_STUN_TIME}, animation::{Animation, AnimateRotation}, stuneffect::{Stun, StunEffect}
};

const TREE_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
const TREE_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -2.0 * SCALE_FACTOR);
const STONE_COLLIDABLE_DIMENSIONS: (f32, f32) = (3.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
const STONE_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -1.0 * SCALE_FACTOR);
const REGION_OFFSETS: [(isize, isize); 5] = [(-1, 0), (-1, -1), (0, -1), (1, -1), (1, 0)];
const GAP_RANGE: (f32, f32) = (SPRITE_SIZE * SCALE_FACTOR * 1.0, SPRITE_SIZE * SCALE_FACTOR * 5.0);


#[derive(Clone)]
enum ObstacleType {
    Stone,
    Tree,
}

struct XYFillSpawner {
    region_width: f32,
    region_height: f32,
    current_filled_x: f32,
    current_filled_y: f32,
    y_gap: f32,
}

fn xy_walk_spawner(region_width: f32, region_height: f32) -> XYFillSpawner {
    let mut rng = rand::thread_rng();
    let y_gap = rng.gen_range(GAP_RANGE.0..GAP_RANGE.1);
    let current_filled_y = y_gap * 2.0;
    XYFillSpawner {
        region_width,
        region_height,
        current_filled_x: rng.gen_range(GAP_RANGE.0..GAP_RANGE.1),
        current_filled_y,
        y_gap,
    }
}

impl Iterator for XYFillSpawner {
    type Item = (ObstacleType, f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_filled_y > self.region_height {
            return None;
        }
        let mut rng = rand::thread_rng();
        let choices = [ObstacleType::Tree, ObstacleType::Stone];
        let Some(obstacle_type) = choices.choose(&mut rng) else {
            return None;
        };

        let data = (obstacle_type.clone(), self.current_filled_x, self.current_filled_y + rng.gen_range(-self.y_gap * 0.75..self.y_gap * 0.75));
        if self.current_filled_x + GAP_RANGE.1 > self.region_width {
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
                    update_collidables.after(player::update_player),
                    process_collisions_player.after(update_collidables),
                    process_collisions_yeti.after(update_collidables),
                    cleanup::<Obstacle>,
                    cleanup_regions.after(spawn_obstales),
                ).in_set(OnUpdate(GameState::Playing))
            );
    }
}


fn spawn_obstales(
    mut commands: Commands,
    window: Query<&Window>,
    game_resources: Res<GameResources>,
    camera_q: Query<&Transform, With<Camera>>,
    mut spawner_r: ResMut<ObstacleSpawner>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let Ok(transform) = camera_q.get_single() else {
        return;
    };
    let region_width = window.width();
    let region_height = window.height();
    let camera_region_x = (transform.translation.x / region_width).floor() as isize;
    let camera_region_y = (transform.translation.y / region_height).floor() as isize;
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

        for (obstacle_type, ox, oy) in xy_walk_spawner(region_width, region_height) {
            let x = (rx as f32 - 0.5) * region_width + ox;
            let y = ry as f32 * region_height - oy;

            let (sprite_rect, collidable_dimension, offsets, offset_z) = match obstacle_type {
                ObstacleType::Tree => (game_resources.tree, TREE_COLLIDABLE_DIMENSIONS, TREE_COLLIDABLE_OFFSETS, 0.2),
                ObstacleType::Stone => (game_resources.stone, STONE_COLLIDABLE_DIMENSIONS, STONE_COLLIDABLE_OFFSETS, 0.1)
            };

            commands.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                        rect: Some(sprite_rect.clone()),
                        ..default()
                    },
                    texture: game_resources.image_handle.clone(),
                    transform: Transform::from_xyz(x, y, PLAYER_Z_INDEX + 1.0 + offset_z),
                    ..default()
                },
                Collidable::new(x, y, collidable_dimension.0, collidable_dimension.1, offsets.0, offsets.1),
                Alive,
                Obstacle,
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
    window: Query<&Window>,
    mut spawner_r: ResMut<ObstacleSpawner>,
) {
    let Ok(transform) = camera_q.get_single() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };
    let region_height = window.height();

    let camera_region_y = (transform.translation.y / region_height).floor() as isize;
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

fn process_collisions_player(
    mut commands: Commands,
    game_resources: Res<GameResources>,
    mut player_q: Query<(Entity, &mut Sprite, &Collidable), (With<Player>, With<Alive>, Without<Obstacle>)>,
    skis_q: Query<Entity, Or<(With<LeftSki>, With<RightSki>)>>,
    mut obstacles_q: Query<&Collidable, (With<Obstacle>, Without<Player>)>
) {
    let Ok((entity, mut sprite, collidable_player)) = player_q.get_single_mut() else {
        return;
    };

    let has_collided = obstacles_q.iter_mut().any(|collidable_obstacle| {
        collidable_obstacle.intersect(&collidable_player)
    });

    if has_collided {
        for ski_entity in &skis_q {
            commands.entity(ski_entity).despawn_recursive();
        }
        commands.entity(entity).remove::<Alive>();
        commands.entity(entity).insert(Slowdown(Timer::from_seconds(FALL_TIMEOUT, TimerMode::Once)));
        sprite.rect = Some(game_resources.fall_down);
    }
}

fn process_collisions_yeti(
    mut commands: Commands,
    game_resources: Res<GameResources>,
    mut yeti_q: Query<(Entity, &mut Animation, &Yeti, &Collidable), (With<Yeti>, With<Alive>, Without<Obstacle>, Without<Stun>)>,
    player_q: Query<(Entity, &Collidable), (With<Player>, With<Alive>, Without<Obstacle>, Without<Yeti>)>,
    mut obstacles_q: Query<&Collidable, (With<Obstacle>, Without<Yeti>)>
) {
    let Ok((
        entity_yeti,
        mut animation,
        yeti,
        collidable_yeti
    )) = yeti_q.get_single_mut() else {
        return;
    };
    if !yeti.ignore_collisions.finished() {
        return;
    }
    let Ok((
        entity_player,
        collidable_player
    )) = player_q.get_single() else {
        return;
    };

    let has_collided_obstacle = obstacles_q.iter_mut().any(|collidable_obstacle| {
        collidable_obstacle.intersect(&collidable_yeti)
    });
    if has_collided_obstacle {
        animation.set_frames(vec![game_resources.yeti_fallen]);
        let stun_child = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                    rect: Some(game_resources.stun),
                    ..default()
                },
                texture: game_resources.image_handle.clone(),
                transform: Transform::from_xyz(0.0, 7.0 * SCALE_FACTOR, 0.5),
                ..default()
            },
            AnimateRotation {
                angular_vel: PI
            },
            StunEffect
        )).id();
        commands.entity(entity_yeti).push_children(&[stun_child]);
        commands.entity(entity_yeti).insert(Stun(Timer::from_seconds(YETI_STUN_TIME, TimerMode::Once)));
        return;
    }

    if collidable_player.intersect(&collidable_yeti) {
        commands.entity(entity_player).remove::<Alive>();
        commands.entity(entity_player).insert(Catched);
    }
}
