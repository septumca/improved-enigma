use std::{time::Duration, f32::consts::{FRAC_PI_2, FRAC_PI_8, FRAC_PI_4}};
use bevy::{prelude::*, math::vec2, window::WindowResolution};
use rand::{self, Rng, seq::SliceRandom};

/*
TODO
- rozdelit kod do pluginov
- pridat reset button a rozne states https://bevy-cheatbook.github.io/programming/states.html
- spravit git
 */

const SPEED: f32 = 50.0;
const TURN_RATE: f32 = 0.15;
const SCORE_RATIO: f32 = 0.05;
const PLAYER_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.0, 3.5);
const TREE_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.5, 3.5);
const STONE_COLLIDABLE_DIMENSIONS: (f32, f32) = (3.0, 3.0);
const FALL_TIMEOUT: f32 = 0.3;
const SIDE_STOP: f32 = 1.2;
const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 640.0;
const SPRITE_SIZE: f32 = 12.0;
const SCALE_FACTOR: f64 = 6.0;
const OBSTACLE_SPAWN_TIME: (f32, f32) = (0.1, 0.6);
const SIDES_MAX_INDEX: usize = 3;

enum AppState {
    MainMenu,
    Playing,
    GameOver,
}

fn main() {
    App::new()
        .insert_resource(Score::new())
        .insert_resource(ObstacleSpawner {
            timer: Timer::from_seconds(OBSTACLE_SPAWN_TIME.0, TimerMode::Repeating),
            last_spawned_y: -SCREEN_HEIGHT / SCALE_FACTOR as f32 * 0.6
        })
        .insert_resource(SpritesRects::new())
        .insert_resource(ClearColor(Color::rgb(1.0, 0.95, 0.95)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(SCREEN_WIDTH, SCREEN_HEIGHT).with_scale_factor_override(SCALE_FACTOR),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_startup_system(setup)
        .add_system(update_player)
        .add_system(movement_slowdown)
        .add_system(input)
        .add_system(update_collidables)
        .add_system(process_collisions)
        .add_system(gameover_detection)
        .add_system(cleanup::<Obstacle>)
        .add_system(cleanup::<Trail>)
        .add_system(spawn_obstales)
        .add_system(update_score)
        .add_system(leave_trail)
        .run();
}

#[derive(Debug, Component)]
enum Direction {
    Down,
    Left(usize),
    Right(usize)
}

impl Direction {
    fn steer_left(&self) -> Self {
        match self {
            Self::Left(x) if x == &SIDES_MAX_INDEX => Self::Left(SIDES_MAX_INDEX),
            Self::Left(x) => Self::Left(x+1),
            Self::Down => Self::Left(0),
            Self::Right(0) => Self::Down,
            Self::Right(x) => Self::Right(x-1),
        }
    }

    fn steer_right(&self) -> Self {
        match self {
            Self::Right(x) if x == &SIDES_MAX_INDEX => Self::Right(SIDES_MAX_INDEX),
            Self::Right(x) => Self::Right(x+1),
            Self::Down => Self::Right(0),
            Self::Left(0) => Self::Down,
            Self::Left(x) => Self::Left(x-1),
        }
    }
}

#[derive(Component)]
struct Speed(f32);

#[derive(Component)]
struct Player {
    turn_rate: Timer
}

impl Player {
    pub fn new(turn_rate_seconds: f32) -> Self {
        Self { turn_rate: Timer::from_seconds(turn_rate_seconds, TimerMode::Once) }
    }
}

#[derive(Component)]
struct Alive;

#[derive(Component)]
struct Slowdown(Timer);

#[derive(Component)]
struct Obstacle;

#[derive(Component)]
struct Trail;

#[derive(Debug, Component)]
struct Collidable {
    width_half: f32,
    height_half: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Collidable {
    pub fn new(x: f32, y: f32, width_half: f32, height_half: f32) -> Self {
        Self {
            width_half,
            height_half,
            left: x - width_half,
            right: x + width_half,
            top: y + height_half,
            bottom: y - height_half
        }
    }

    pub fn update_center(&mut self, x: f32, y: f32) {
        self.left = x - self.width_half;
        self.right = x + self.width_half;
        self.top = y + self.height_half;
        self.bottom = y - self.height_half;
    }

    pub fn intersect(&self, other: &Collidable) -> bool {
        !(self.right < other.left || other.right < self.left ||
        self.bottom > other.top || other.bottom > self.top)
    }
}

#[derive(Resource)]
struct ObstacleSpawner {
    timer: Timer,
    last_spawned_y: f32,
}

#[derive(Resource)]
struct Score {
    value: f32
}

impl Score {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }
}

#[derive(Resource)]
struct SpritesRects {
    image_handle: Option<Handle<Image>>,
    sprite_size: f32,
    down: Rect,
    sides: Vec<Rect>,
    fall_down: Rect,
    tree: Rect,
    stone: Rect,
}

impl SpritesRects {
    pub fn new() -> Self {
        let sprite_size = SPRITE_SIZE;
        Self {
            image_handle: None,
            sprite_size,
            down: Rect::new(0. * sprite_size, 0., 1. * sprite_size, sprite_size),
            sides: vec![
                Rect::new(1. * sprite_size, 0., 2. * sprite_size, sprite_size),
                Rect::new(2. * sprite_size, 0., 3. * sprite_size, sprite_size),
                Rect::new(3. * sprite_size, 0., 4. * sprite_size, sprite_size),
                Rect::new(4. * sprite_size, 0., 5. * sprite_size, sprite_size),
            ],
            fall_down: Rect::new(5. * sprite_size, 0., 6. * sprite_size, sprite_size),
            tree: Rect::new(6. * sprite_size, 0., 7. * sprite_size, sprite_size),
            stone: Rect::new(7. * sprite_size, 0., 8. * sprite_size, sprite_size),
        }
    }
}

#[derive(Component)]
struct ScoreText {}

fn setup(
    mut commands: Commands,
    mut sprite_rects: ResMut<SpritesRects>,
    asset_server: Res<AssetServer>
) {
    let v1 = vec2(189.5, 78.5).normalize();

    println!("{v1}");

    let texture_handle = asset_server.load("spritesheet.png");
    let font = asset_server.load("QuinqueFive.ttf");
    let text_style = TextStyle {
        font: font.clone(),
        font_size: 4.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Right;
    sprite_rects.image_handle = Some(texture_handle.clone());

    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(vec2(sprite_rects.sprite_size, sprite_rects.sprite_size)),
                rect: Some(sprite_rects.down),
                ..default()
            },
            texture: texture_handle.clone(),
            transform: Transform::from_xyz(0., 0., 1.),
            ..default()
        },
        Player::new(TURN_RATE),
        Alive {},
        Collidable::new(0., 0., PLAYER_COLLIDABLE_DIMENSIONS.0, PLAYER_COLLIDABLE_DIMENSIONS.1),
        Speed(SPEED),
        Direction::Down
    ))
    // .with_children(|parent| {
    //     parent.spawn(SpriteBundle {
    //         sprite: Sprite {
    //             color: Color::rgb(0.25, 0.25, 0.75),
    //             custom_size: Some(Vec2::new(PLAYER_COLLIDABLE_DIMENSIONS.0 * 2.0, PLAYER_COLLIDABLE_DIMENSIONS.1 * 2.0)),
    //             ..default()
    //         },
    //         ..default()
    //     });
    // })
    ;

    commands.spawn((
        TextBundle::from_section(
                "Score: 0",
                text_style,
        )
        .with_text_alignment(text_alignment)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                left: Val::Px(1.),
                top: Val::Px(1.),
                ..default()
            },
            ..default()
        }),
        ScoreText {},
    ));
}

fn input(
    keyboard_input: Res<Input<KeyCode>>,
    sprite_rects: Res<SpritesRects>,
    mut player_q: Query<(&mut Sprite, &mut Direction, &mut Player), With<Alive>>
) {
    let Ok((mut sprite, mut direction, mut player)) = player_q.get_single_mut() else {
        return;
    };

    let mut key_pressed = false;
    if keyboard_input.pressed(KeyCode::A) && player.turn_rate.finished() {
        *direction = direction.steer_left();
        key_pressed = true;
    }

    if keyboard_input.pressed(KeyCode::D) && player.turn_rate.finished() {
        *direction = direction.steer_right();
        key_pressed = true;
    }

    if key_pressed {
        player.turn_rate.reset();
        match *direction {
            Direction::Down => {
                sprite.rect = Some(sprite_rects.down);
                sprite.flip_x = false;
            },
            Direction::Left(x) => {
                sprite.rect = Some(sprite_rects.sides[x]);
                sprite.flip_x = true;
            },
            Direction::Right(x) => {
                sprite.rect = Some(sprite_rects.sides[x]);
                sprite.flip_x = false;
            }
        };
    }
}

fn update_player(
    timer: Res<Time>,
    mut score: ResMut<Score>,
    mut player_q: Query<(&mut Transform, &mut Speed, Option<&mut Slowdown>, &mut Player, &Direction), Without<Camera>>,
    mut camera_q: Query<&mut Transform, With<Camera>>
) {
    let Ok((mut player_transform, speed, slowdown, mut player, direction)) = player_q.get_single_mut() else {
        return;
    };
    let dt = timer.delta();
    player.turn_rate.tick(dt);

    if let Some(mut slowdown) = slowdown {
        if slowdown.0.tick(dt).finished() {
            return;
        }
    }

    let (deg_rad, speed_modifier) = match direction {
        Direction::Down => (0.0, 1.0),
        Direction::Left(0) => (-FRAC_PI_8, 0.85), //22.5
        Direction::Right(0) =>  (FRAC_PI_8, 0.85), //22.5
        Direction::Left(1) => (-FRAC_PI_4, 0.8), //45
        Direction::Right(1) => (FRAC_PI_4, 0.8), //45
        Direction::Left(2) => (-FRAC_PI_8 * 3.0, 0.7), //77.5
        Direction::Right(2) => (FRAC_PI_8 * 3.0, 0.7), //77.5
        Direction::Left(3) => (-FRAC_PI_2, 0.55), //90
        Direction::Right(3) => (FRAC_PI_2, 0.55), //90
        _ => (0.0, 0.0)
    };
    let deg_rad = deg_rad - FRAC_PI_2; //0 degrees is pointing down (e.g. [0, -1], not to [1, 0])

    let dx = deg_rad.cos() * speed.0 * speed_modifier;
    let dy = deg_rad.sin() * speed.0 * speed_modifier;
    player_transform.translation.x += dx * dt.as_secs_f32();
    player_transform.translation.y += dy * dt.as_secs_f32();
    score.value += dy.abs() / SPEED * SCORE_RATIO ;

    let Ok(mut camera_transform) = camera_q.get_single_mut() else {
        return;
    };
    camera_transform.translation = player_transform.translation;
}

fn leave_trail(
    mut commands: Commands,
    player_q: Query<(&Transform, Option<&Slowdown>, &Direction), (With<Player>, With<Alive>)>,
) {
    let Ok((transform, slowdown, direction)) = player_q.get_single() else {
        return;
    };
    if let Some(slowdown) = slowdown {
        if slowdown.0.finished() {
            return;
        }
    }

    let offsets = match direction {
        Direction::Down | Direction::Right(0) | Direction::Left(0) => vec![(-2., -1.), (2., -1.)],
        Direction::Right(1) | Direction::Left(1) => vec![(-2., -2.), (2., -2.)],
        Direction::Right(2) | Direction::Left(2) => vec![(-2., -3.), (2., -3.)],
        Direction::Right(3) => vec![(-2., -4.), (2., -3.)],
        Direction::Left(3) => vec![(-2., -3.), (2., -4.)],
        _ => vec![]
    };

    for (dx, dy) in offsets {
        commands.spawn(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.38, 0.34, 0.31),
                custom_size: Some(Vec2::new(1., 1.)),
                ..default()
            },
            transform: Transform::from_xyz(transform.translation.x + dx, transform.translation.y + dy, 0.),
            ..default()
        });
    }
}

fn movement_slowdown(
    mut commands: Commands,
    player_q: Query<(Entity, &Direction), (Changed<Direction>, With<Player>)>
) {
    let Ok((entity, direction)) = player_q.get_single() else {
        return;
    };

    commands.entity(entity).remove::<Slowdown>();
    match direction {
        Direction::Left(3) | Direction::Right(3) => {
            commands.entity(entity).insert(Slowdown(Timer::from_seconds(SIDE_STOP, TimerMode::Once)));
        },
        _ => ()
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
    sprite_rects: Res<SpritesRects>,
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

fn spawn_obstales(
    mut commands: Commands,
    timer: Res<Time>,
    sprite_rects: Res<SpritesRects>,
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

fn update_score(
    score: Res<Score>,
    mut text_q: Query<&mut Text, With<ScoreText>>
) {
    if !score.is_changed() {
        return;
    }
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };
    text.sections[0].value = format!("Score: {:.0}", score.value);
}

fn cleanup<T: Component>(
    mut commands: Commands,
    camera_q: Query<&Transform, With<Camera>>,
    obstacles_q: Query<(Entity, &Transform), With<T>>
) {
    let Ok(transform_camera) = camera_q.get_single() else {
        return;
    };
    let offset = SCREEN_HEIGHT / SCALE_FACTOR as f32 * 0.6;

    for (entity, transform_obstacle) in obstacles_q.iter() {
        if transform_obstacle.translation.y - transform_camera.translation.y > offset {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn gameover_detection(
    timer_q: Query<&Slowdown, (With<Player>, Without<Alive>)>,
) {
    let Ok(slowdown) = timer_q.get_single() else {
        return;
    };
    if slowdown.0.just_finished() {
        println!("GAME OVER");
        //GAME OVER
    }
}