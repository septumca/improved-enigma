use bevy::{prelude::*, math::vec2};
use rand::{self, Rng};

use crate::{
    Alive,
    GameState,
    SCALE_FACTOR,
    GameResources,
    player::{
        self,
        Player,
        Score,
        PLAYER_Z_INDEX
    },
    cleanup,
    despawn,
    SPRITE_SIZE, sounds::PostHitEvent
};

const GAP_RANGE_X : (f32, f32) = (SPRITE_SIZE * SCALE_FACTOR * 2.5, SPRITE_SIZE * SCALE_FACTOR * 6.0);
const GAP_RANGE_Y: (f32, f32) = (SPRITE_SIZE * SCALE_FACTOR * 4.0, SPRITE_SIZE * SCALE_FACTOR * 5.0);
const POST_DISTANCE: f32 = SPRITE_SIZE * SCALE_FACTOR * 3.5;
const FIRST_POST_DISTANCE: f32 = 7.0 * SPRITE_SIZE * SCALE_FACTOR;
const HIT_DETECTION_OFFSET: (f32, f32) = (10.0, 0.0);

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


#[derive(Resource)]
struct PostsOrder {
    posts: Vec<PostsData>,
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


#[derive(Clone)]
struct PostsData {
    x_left: f32,
    x_right: f32,
    y: f32,
}

#[derive(Component)]
struct Posts;

pub struct PostsPlugin;

impl Plugin for PostsPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(PostsSpawner {
                x: 0.0, y: 0.0, color: PostColor::Blue
            })
            .insert_resource(PostsOrder {
                posts: vec![]
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
    mut posts_ordered: ResMut<PostsOrder>,
) {
    spawner_r.x = 0.0;
    spawner_r.y = 0.0;
    spawner_r.color = PostColor::Blue;
    posts_ordered.posts = vec![];
}

fn spawn_posts(
    mut commands: Commands,
    window: Query<&Window>,
    camera_q: Query<&Transform, With<Camera>>,
    game_resources: Res<GameResources>,
    mut posts_ordered: ResMut<PostsOrder>,
    mut spawner_r: ResMut<PostsSpawner>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };
    let Ok(transform) = camera_q.get_single() else {
        return;
    };
    if (spawner_r.y - transform.translation.y).abs() > window.height() {
        return;
    }
    let Some((color, x, y)) = spawner_r.next() else {
        return;
    };

    posts_ordered.posts.push(PostsData {
        x_left: x - POST_DISTANCE / 2.0,
        x_right: x + POST_DISTANCE / 2.0,
        y
    });
    commands.spawn((
        SpatialBundle {
            transform: Transform::from_xyz(x, y, PLAYER_Z_INDEX + 1.1),
            ..default()
        },
        Alive,
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
}

fn detect_posts_hit(
    mut posts_ordered: ResMut<PostsOrder>,
    mut ev_posthit: EventWriter<PostHitEvent>,
    mut player_q: Query<(&Transform, &mut Score), (With<Alive>, With<Player>)>,
) {
    let Ok((transform, mut score)) = player_q.get_single_mut() else {
        return;
    };
    let Some(post) = posts_ordered.posts.get(0) else {
        return;
    };

    if post.y + HIT_DETECTION_OFFSET.0 >= transform.translation.y &&
        post.y + HIT_DETECTION_OFFSET.1 <= transform.translation.y &&
        post.x_left <= transform.translation.x &&
        post.x_right >= transform.translation.x
    {
        // println!("HIT");
        // println!("PLAYER: {}, {}", transform.translation.x, transform.translation.y);
        // println!("POST Y: {}", post.y);
        // println!("POST X: {}, {}", post.x_left, post.x_right);
        score.increase();
        posts_ordered.posts.remove(0);
        ev_posthit.send(PostHitEvent);
    }

    let Some(post) = posts_ordered.posts.get(0) else {
        return;
    };
    if post.y + HIT_DETECTION_OFFSET.1 - 2.0 > transform.translation.y {
        // println!("MISS");
        // println!("PLAYER: {}, {}", transform.translation.x, transform.translation.y);
        // println!("POST Y: {}", post.y);
        // println!("POST X: {}, {}", post.x_left, post.x_right);
        score.decrease();
        posts_ordered.posts.remove(0);
    }

}
