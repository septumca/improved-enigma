use std::f32::consts::PI;

use bevy::{prelude::*, math::vec2, utils::HashSet};
use rand::{self, Rng, seq::SliceRandom};

use crate::{
    collidable::{Collidable, CollidableMovable},
    Alive,
    GameState,
    SCALE_FACTOR,
    GameResources,
    player::{
        Player,
        Falldown,
        FALL_TIMEOUT,
        PLAYER_Z_INDEX, LeftSki, RightSki, self, Catched, Slowdown
    },
    cleanup,
    despawn,
    debug::DebugMarker,
    SPRITE_SIZE, yeti::{Yeti, YETI_STUN_TIME}, animation::{Animation, AnimateRotation}, stuneffect::{Stun, StunEffect}
};

pub const TREE_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
pub const TREE_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -2.0 * SCALE_FACTOR);
pub const STONE_COLLIDABLE_DIMENSIONS: (f32, f32) = (3.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
pub const STONE_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -1.0 * SCALE_FACTOR);
pub const REGION_OFFSETS: [(isize, isize); 5] = [
    (-1, 0), (-1, -1),
    (0, -1),
    (1, -1), (1, 0)
];
pub const GAP_RANGE: (f32, f32) = (SPRITE_SIZE * SCALE_FACTOR * 1.0, SPRITE_SIZE * SCALE_FACTOR * 5.0);
pub const DIFFICULTY_RAMPUP: f32 = 0.03;

#[derive(Clone)]
pub enum ObstacleType {
    Stone,
    Tree,
}

struct TestSpawner {
    x: f32,
    y: f32,
    count: usize,
}

impl TestSpawner {
    fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            count: 0,
        }
    }
}

struct XYFillSpawner {
    region_width: f32,
    region_height: f32,
    current_filled_x: f32,
    current_filled_y: f32,
    y_gap: f32,
}

impl XYFillSpawner {
    fn new(region_width: f32, region_height: f32) -> Self {
        let mut rng = rand::thread_rng();
        let y_gap = rng.gen_range(GAP_RANGE.0..GAP_RANGE.1);
        let current_filled_y = y_gap * 2.0;
        Self {
            region_width,
            region_height,
            current_filled_x: rng.gen_range(GAP_RANGE.0..GAP_RANGE.1),
            current_filled_y,
            y_gap,
        }
    }
}

struct TileSpawner {
    tiles_count: usize,
    width: usize,
    act_tile: usize,
    tile_size: f32,
    spawn_offset: (f32, f32),
    spawn_chance: f32
}

impl TileSpawner {
    fn new(
        tiles_count: usize,
        width: usize,
        tile_size: f32,
        spawn_offset: (f32, f32),
        spawn_chance: f32
    ) -> Self {
        Self {
            tiles_count,
            width,
            act_tile: 0,
            tile_size,
            spawn_offset,
            spawn_chance
        }
    }
}

impl Iterator for TestSpawner {
    type Item = (ObstacleType, f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;
        if self.count < 5 {
            return Some((ObstacleType::Tree, self.x, self.y - 300.0 * (self.count + 1) as f32));
        }
        None
    }
}

impl Iterator for TileSpawner {
    type Item = (ObstacleType, f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        let mut rng = rand::thread_rng();
        let mut spawn_chance = rng.gen_range(0.0..1.0);

        while spawn_chance > self.spawn_chance {
            self.act_tile += 1;
            if self.act_tile > self.tiles_count {
                return None;
            }
            spawn_chance = rng.gen_range(0.0..1.0);
        }
        if self.act_tile > self.tiles_count {
            return None;
        }

        // info!("SPAWNING {} from {}", self.act_tile, self.tiles_count);
        let x = (self.act_tile % self.width) as f32 * self.tile_size + rng.gen_range(-self.spawn_offset.0..self.spawn_offset.0);
        let y = (self.act_tile / self.width) as f32 * self.tile_size + rng.gen_range(-self.spawn_offset.1..self.spawn_offset.1);
        let choices = [ObstacleType::Tree, ObstacleType::Stone];
        let Some(obstacle_type) = choices.choose(&mut rng) else {
            return None;
        };
        self.act_tile += 1;
        Some((obstacle_type.clone(), x, y))
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
pub struct ObstacleSpawner {
    last_processed_player_region_y: Option<isize>,
    last_processed_player_region_x: Option<isize>,
    spawned_regions: HashSet<(isize, isize)>,
}

#[derive(Component)]
pub struct Obstacle;

pub struct ObstaclePlugin;

impl Plugin for ObstaclePlugin {
    fn build(&self, app: &mut App) {
        app
            // .insert_resource(ObstacleSpawner {
            //     last_processed_player_region_y: None,
            //     last_processed_player_region_x: None,
            //     spawned_regions: HashSet::new(),
            // })
            .add_system(despawn::<Obstacle>.in_schedule(OnExit(GameState::GameOver)))
            .add_system(reset_spawner.in_schedule(OnExit(GameState::GameOver)))
            .add_systems(
                (
                    spawn_obstacles,
                    update_collidables.after(player::update_player),
                    process_collisions_player.after(update_collidables),
                    process_collisions_yeti.after(update_collidables),
                    cleanup::<Obstacle>,
                    cleanup_regions.after(spawn_obstacles),
                    // cleanup_spatial_tree
                ).in_set(OnUpdate(GameState::Playing))
            )
            ;
    }
}


pub fn spawn_obstacles(
    mut commands: Commands,
    window: Query<&Window>,
    game_resources: Res<GameResources>,
    camera_q: Query<&Transform, With<Camera>>,
    spawner_r: Option<ResMut<ObstacleSpawner>>,
) {
    let Some(mut spawner_r) = spawner_r else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };
    let Ok(transform) = camera_q.get_single() else {
        return;
    };
    info!("SPAWNING");
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

        // let spawner = TestSpawner::new(transform.translation.x + region_width, transform.translation.y);
        let tile_size = 60.0;
        let height = (region_height / tile_size) as usize;
        let width = (region_width / tile_size) as usize + 1;
        let spawner = TileSpawner::new(
            height * width,
            width,
            tile_size,
            (15., 15.),
            (camera_region_y.abs() as f32).log2() * DIFFICULTY_RAMPUP);
        // let spawner = SpawnerType::XYGaps => XYFillSpawner::new(region_width, region_height);
        for (obstacle_type, ox, oy) in spawner {
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

pub fn cleanup_regions(
    camera_q: Query<&Transform, With<Camera>>,
    window: Query<&Window>,
    spawner_r: Option<ResMut<ObstacleSpawner>>,
) {
    let Some(mut spawner_r) = spawner_r else {
        return;
    };
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
    spawner_r: Option<ResMut<ObstacleSpawner>>
) {
    let Some(mut spawner_r) = spawner_r else {
        return;
    };
    spawner_r.last_processed_player_region_y = None;
    spawner_r.last_processed_player_region_x = None;
    spawner_r.spawned_regions = HashSet::new();
}

pub fn update_collidables(
    mut collidables_q: Query<(&Transform, &mut Collidable), With<CollidableMovable>>
) {
    for (transform, mut collidable) in collidables_q.iter_mut() {
        collidable.update_center(transform.translation.x, transform.translation.y);
    }
}

pub fn process_collisions_player(
    mut commands: Commands,
    game_resources: Res<GameResources>,
    mut player_q: Query<(Entity, &mut Sprite, &Collidable, &Children), (With<Player>, With<Alive>, Without<Obstacle>)>,
    mut skis_q: Query<&mut Visibility, Or<(With<LeftSki>, With<RightSki>)>>,
    obstacles_q: Query<&Collidable, (With<Obstacle>, Without<Player>)>
) {
    let Ok((
        entity,
        mut sprite,
        collidable_player,
        children
    )) = player_q.get_single_mut() else {
        return;
    };
    let has_collided = obstacles_q.iter().any(|collidable_obstacle| {
        collidable_obstacle.intersect(&collidable_player)
    });

    if has_collided {
        for &ch in children {
            if let Ok(mut visibility) = skis_q.get_mut(ch) {
                *visibility = Visibility::Hidden;
            }
        }
        commands.entity(entity).insert(Slowdown(Timer::from_seconds(FALL_TIMEOUT, TimerMode::Once)));
        commands.entity(entity).insert(Falldown);
        sprite.rect = Some(game_resources.fall_down);
    }
}

pub fn process_collisions_yeti(
    mut commands: Commands,
    game_resources: Res<GameResources>,
    mut yeti_q: Query<(Entity, &mut Animation, &Yeti, &Collidable), (With<Yeti>, With<Alive>, Without<Obstacle>, Without<Stun>)>,
    player_q: Query<(Entity, &Collidable), (With<Player>, With<Alive>, Without<Obstacle>, Without<Yeti>)>,
    obstacles_q: Query<&Collidable, (With<Obstacle>, Without<Yeti>)>,
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
    let has_collided = obstacles_q.iter().any(|collidable_obstacle| {
        collidable_obstacle.intersect(&collidable_yeti)
    });
    if has_collided {
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

    let Ok((
        entity_player,
        collidable_player
    )) = player_q.get_single() else {
        return;
    };
    if collidable_player.intersect(&collidable_yeti) {
        commands.entity(entity_player).remove::<Alive>();
        commands.entity(entity_player).insert(Catched);
    }
}
