use std::{f32::consts::{FRAC_PI_2, FRAC_PI_8, FRAC_PI_4}};
use bevy::{prelude::*, math::vec2};

use crate::{GameResources, Alive, GameState, collidable::Collidable, despawn, debug::{DebugMarker}, SCALE_FACTOR};


const SPEED: f32 = 50.0 * SCALE_FACTOR;
const TURN_RATE: f32 = 0.15;
const PLAYER_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
const PLAYER_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -2.0 * SCALE_FACTOR);
pub const FALL_TIMEOUT: f32 = 0.3;
const SIDES_MAX_INDEX: usize = 3;
const TURN_ANGLES: [f32; 4] = [FRAC_PI_8, FRAC_PI_4, FRAC_PI_8 * 3.0, FRAC_PI_2];


#[derive(Debug, Component)]
pub enum Direction {
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
pub struct SpeedModifiers {
    down: f32,
    side: Vec<f32>,
}

#[derive(Component)]
pub struct Player {
    turn_rate: Timer
}

impl Player {
    pub fn new(turn_rate_seconds: f32) -> Self {
        Self {
            turn_rate: Timer::from_seconds(turn_rate_seconds, TimerMode::Once)
        }
    }
}


#[derive(Component)]
pub struct Slowdown(pub Timer);


#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameOverText;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(setup.in_schedule(OnEnter(GameState::Playing)))
            .add_systems(
                (
                    despawn::<Player>,
                    despawn::<ScoreText>,
                ).in_schedule(OnExit(GameState::GameOver))
            )
            .add_systems(
                (
                    input,
                    update_player.after(input),
                    gameover_detection.after(update_player),
                ).in_set(OnUpdate(GameState::Playing))
            );
    }
}


fn input(
    keyboard_input: Res<Input<KeyCode>>,
    sprite_rects: Res<GameResources>,
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
    mut player_q: Query<(&mut Transform, &mut Speed, &mut Player, &Direction, &SpeedModifiers), Without<Camera>>,
    mut camera_q: Query<&mut Transform, With<Camera>>
) {
    let Ok((
        mut player_transform,
        speed,
        mut player,
        direction,
        speed_modifiers,
    )) = player_q.get_single_mut() else {
        return;
    };
    let dt = timer.delta();
    player.turn_rate.tick(dt);

    let (deg_rad, speed_modifier) = match direction {
        Direction::Down => (0.0, speed_modifiers.down),
        Direction::Left(x) => (-TURN_ANGLES[*x], speed_modifiers.side[*x]),
        Direction::Right(x) => (TURN_ANGLES[*x], speed_modifiers.side[*x]),
    };
    let deg_rad = deg_rad - FRAC_PI_2; //0 degrees is pointing down (e.g. [0, -1], not to [1, 0])
    let dx = deg_rad.cos() * speed.0 * speed_modifier;
    let dy = deg_rad.sin() * speed.0 * speed_modifier;

    player_transform.translation.x += dx * dt.as_secs_f32();
    player_transform.translation.y += dy * dt.as_secs_f32();

    let Ok(mut camera_transform) = camera_q.get_single_mut() else {
        return;
    };
    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y;
}

fn gameover_detection(
    time: Res<Time>,
    mut timer_q: Query<&mut Slowdown, (With<Player>, Without<Alive>)>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    let Ok(mut slowdown) = timer_q.get_single_mut() else {
        return;
    };
    if slowdown.0.tick(time.delta()).just_finished() {
        app_state.set(GameState::GameOver)
    }
}

pub fn setup(
    mut commands: Commands,
    game_resources: Res<GameResources>,
) {
    let Some(font_handle) = &game_resources.font_handle else {
        return;
    };
    let Some(image_handle) = &game_resources.image_handle else {
        return;
    };

    let text_style = TextStyle {
        font: font_handle.clone(),
        font_size: 24.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Right;

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                rect: Some(game_resources.down),
                ..default()
            },
            texture: image_handle.clone(),
            transform: Transform::from_xyz(0., 0., 2.),
            ..default()
        },
        Player::new(TURN_RATE),
        Alive,
        Collidable::new(
            0., 0.,
            PLAYER_COLLIDABLE_DIMENSIONS.0, PLAYER_COLLIDABLE_DIMENSIONS.1,
            PLAYER_COLLIDABLE_OFFSETS.0, PLAYER_COLLIDABLE_OFFSETS.1
        ),
        Speed(SPEED),
        SpeedModifiers {
            down: 1.0,
            side: vec![0.85, 0.75, 0.6, 0.4]
        },
        Direction::Down
    ))
    .with_children(|parent| {
        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::rgb(0.25, 0.25, 0.75),
                    custom_size: Some(Vec2::new(PLAYER_COLLIDABLE_DIMENSIONS.0 * 2.0, PLAYER_COLLIDABLE_DIMENSIONS.1 * 2.0)),
                    ..default()
                },
                transform: Transform::from_xyz(PLAYER_COLLIDABLE_OFFSETS.0, PLAYER_COLLIDABLE_OFFSETS.1, 1.0),
                visibility: Visibility::Hidden,
                ..default()
            },
            DebugMarker
        ));
    })
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
        ScoreText,
    ));
}
