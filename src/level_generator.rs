use bevy::{prelude::*, math::vec2};
use rand::{Rng, seq::SliceRandom};

use crate::{GameState, GameResources, obstacle::{ObstacleType, TREE_COLLIDABLE_DIMENSIONS, TREE_COLLIDABLE_OFFSETS, STONE_COLLIDABLE_DIMENSIONS, STONE_COLLIDABLE_OFFSETS, Obstacle}, player::{self, PLAYER_Z_INDEX}, collidable::Collidable, Alive, debug::DebugMarker, finish::Finish, posts::{PostsSpawner, PostColor, POST_DISTANCE, HIT_DETECTION_OFFSET, Posts}};

#[derive(Resource)]
pub struct LevelGeneratorSettings {
    pub tile_size: f32,
    pub displacement: f32,
    pub start_offset_y: f32,
    pub width: usize,
    pub height: usize,
    pub starting_difficulty: f32,
}

pub struct LevelGeneratorPlugin;


impl Plugin for LevelGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(generate.after(player::setup).in_schedule(OnEnter(GameState::Playing)));
    }
}

fn generate(
    mut commands: Commands,
    game_resources: Res<GameResources>,
    levelgenerator: Option<Res<LevelGeneratorSettings>>
) {
    let Some(levelgenerator) = levelgenerator else {
        return;
    };
    let mut rng = rand::thread_rng();
    let width_halved = (levelgenerator.width / 2) as isize;

    for ty in 0..levelgenerator.height as isize {
        for tx in -width_halved..width_halved {
            let spawn_change = rng.gen_range(0.0..=1.0);
            if spawn_change > levelgenerator.starting_difficulty {
                continue;
            }
            let displacement_x = rng.gen_range(-levelgenerator.displacement..levelgenerator.displacement);
            let displacement_y = rng.gen_range(-levelgenerator.displacement..levelgenerator.displacement);
            let x = tx as f32 * levelgenerator.tile_size + displacement_x;
            let y = -ty as f32 * levelgenerator.tile_size + displacement_y + levelgenerator.start_offset_y;
            let choices = [ObstacleType::Tree, ObstacleType::Stone];
            let Some(obstacle_type) = choices.choose(&mut rng) else {
                continue;
            };

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
    }
    let finish_line_y = -(levelgenerator.height as f32) * levelgenerator.tile_size + levelgenerator.start_offset_y + 100.0;
    let post_spawner =  PostsSpawner::new();
    let max_post_y = finish_line_y + 300.0;
    for (color, x, y) in post_spawner {
        commands.spawn((
            SpatialBundle {
                transform: Transform::from_xyz(x, y, PLAYER_Z_INDEX + 1.1),
                ..default()
            },
            Alive,
            Collidable::new(x, y, (POST_DISTANCE - 1.0) / 2.0, HIT_DETECTION_OFFSET / 2.0, 0.0, HIT_DETECTION_OFFSET / 2.0),
            Posts
        ))
        .with_children(|parent| {
            let sprite_rect = match color {
                PostColor::Blue => game_resources.blue_post,
                PostColor::Red => game_resources.red_post,
            };

            parent.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                        rect: Some(sprite_rect.clone()),
                        ..default()
                    },
                    texture: game_resources.image_handle.clone(),
                    transform: Transform::from_xyz(-POST_DISTANCE / 2.0, 0.0, 0.),
                    ..default()
                },
            ));
            parent.spawn((
                SpriteBundle {
                    sprite: Sprite {
                        custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                        rect: Some(sprite_rect.clone()),
                        ..default()
                    },
                    texture: game_resources.image_handle.clone(),
                    transform: Transform::from_xyz(POST_DISTANCE / 2.0, 0.0, 0.),
                    ..default()
                },
            ));
        });

        if max_post_y > y {
            break;
        }
    }

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::BLACK,
                custom_size: Some(Vec2::new(
                    levelgenerator.width as f32 * levelgenerator.tile_size,
                    8.0
                )),
                ..default()
            },
            transform: Transform::from_xyz(0.0, finish_line_y, 0.0),
            ..default()
        },
        Finish(finish_line_y),
    ));
}