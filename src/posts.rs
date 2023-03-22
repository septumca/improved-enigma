use bevy::{prelude::*, math::vec2};
use rand::{self, Rng};

use crate::{Alive, GameState, SCREEN_HEIGHT, SCALE_FACTOR, GameResources, player::{self, Player}, cleanup, despawn, SPRITE_SIZE};

const MAX_SPAWN_DISTANCE: f32 = SCREEN_HEIGHT * 3.0;
const GAP_RANGE_X : (f32, f32) = (SPRITE_SIZE * SCALE_FACTOR * 1.0, SPRITE_SIZE * SCALE_FACTOR * 4.0);
const GAP_RANGE_Y: (f32, f32) = (SPRITE_SIZE * SCALE_FACTOR * 4.0, SPRITE_SIZE * SCALE_FACTOR * 5.0);
const POST_DISTANCE: f32 = SPRITE_SIZE * SCALE_FACTOR * 3.5;
const FIRST_POST_DISTANCE: f32 = 5.0 * SPRITE_SIZE * SCALE_FACTOR;

#[derive(Clone)]
enum PostColor {
    Blue,
    Red,
}

#[derive(Resource)]
struct PostsSpawner {
    x: f32,
    y: f32,
    color: PostColor
}

impl Iterator for PostsSpawner {
    type Item = (PostColor, f32, f32);

    fn next(&mut self) -> Option<Self::Item> {
        if self.y == 0.0 {
            self.y = -FIRST_POST_DISTANCE;
            return Some((self.color.clone(), self.x, self.y))
        }
        let mut rng = rand::thread_rng();
        let (color, signum) = match self.color {
            PostColor::Blue => (PostColor::Red, 1.0),
            PostColor::Red => (PostColor::Blue, -1.0)
        };

        self.x = self.x + rng.gen_range(GAP_RANGE_X.0..GAP_RANGE_X.1) * signum;
        self.y = self.y - rng.gen_range(GAP_RANGE_Y.0..GAP_RANGE_Y.1);
        self.color = color;

        Some((self.color.clone(), self.x, self.y))
    }
}


#[derive(Component)]
struct Posts {
    x_left: f32,
    x_right: f32,
    y: f32,
}

pub struct PostsPlugin;

impl Plugin for PostsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PostsSpawner {
                x: 0.0, y: 0.0, color: PostColor::Blue
            })
            .add_system(setup.after(player::setup).in_schedule(OnEnter(GameState::Playing)))
            .add_system(despawn::<Posts>.in_schedule(OnExit(GameState::GameOver)))
            .add_systems(
                (
                    spawn_posts,
                    detect_posts_hit,
                    cleanup::<Posts>,
                ).in_set(OnUpdate(GameState::Playing))
            );
    }
}


fn setup(
    mut spawner_r: ResMut<PostsSpawner>,
) {
    spawner_r.x = 0.0;
    spawner_r.y = 0.0;
    spawner_r.color = PostColor::Blue;
}


fn spawn_posts(
    mut commands: Commands,
    camera_q: Query<&Transform, With<Camera>>,
    game_resources: Res<GameResources>,
    mut spawner_r: ResMut<PostsSpawner>,
) {
    let Ok(transform) = camera_q.get_single() else {
        return;
    };
    let Some(texture_handle) = &game_resources.image_handle else {
        return;
    };
    let Some(font_handle) = &game_resources.font_handle else {
        return;
    };
    if (spawner_r.y - transform.translation.y).abs() > MAX_SPAWN_DISTANCE {
        return;
    }

    let Some((color, x, y)) = spawner_r.next() else {
        return;
    };

    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(x, y, 0.),
            ..default()
        },
        Alive,
        Posts {
            x_left: x - POST_DISTANCE / 2.0,
            x_right: x + POST_DISTANCE / 2.0,
            y
        }
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
                texture: texture_handle.clone(),
                transform: Transform::from_xyz(x - POST_DISTANCE / 2.0, 0.0, 0.),
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
                texture: texture_handle.clone(),
                transform: Transform::from_xyz(x + POST_DISTANCE / 2.0, 0.0, 0.),
                ..default()
            },
        ));
    });
}

fn detect_posts_hit(
    posts_q: Query<&Posts>,
    player_q: Query<&Transform, (With<Player>, With<Alive>)>,
) {
}
