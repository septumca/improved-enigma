use std::{f32::consts::{FRAC_PI_2, FRAC_PI_8, FRAC_PI_4, PI}};
use bevy::{prelude::*, math::vec2};

use crate::{GameResources, Alive, GameState, collidable::Collidable, despawn, debug::{DebugMarker}, SCALE_FACTOR};


const SPEED: f32 = 50.0 * SCALE_FACTOR;
const TURN_RATE: f32 = 0.15;
const PLAYER_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
const PLAYER_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -2.0 * SCALE_FACTOR);
pub const FALL_TIMEOUT: f32 = 0.3;
const SIDES_MAX_INDEX: usize = 3;
const TURN_ANGLES: [f32; 4] = [FRAC_PI_8, FRAC_PI_4, FRAC_PI_8 * 3.0, FRAC_PI_2];
pub const PLAYER_Z_INDEX: f32 = 2.0;
const PLAYER_CAMERA_OFFSET: f32 = 128.0;

#[derive(Debug, Component)]
pub enum Direction {
    Down,
    Left(usize),
    Right(usize)
}

enum SteerDirection {
    Left,
    Right
}

impl Direction {
    pub fn from_angle(angle: f32) -> Self {
        let treshold = FRAC_PI_8 / 2.0;
        let is_right_side = angle > 0.0 && angle < PI;
        let angle = angle.abs();
        if angle <= treshold {
            return Self::Down;
        }
        let mut side_index = SIDES_MAX_INDEX;
        if angle > treshold && angle <= treshold * 3.0 {
            side_index = 0;
        }
        if angle > treshold * 3.0 && angle <= treshold * 5.0 {
            side_index = 1;
        }
        if angle > treshold * 5.0 && angle <= treshold * 7.0 {
            side_index = 2;
        }
        match is_right_side {
            true => Self::Right(side_index),
            false => Self::Left(side_index)
        }
    }

    fn steering_direction(&self, target: &Self) -> Option<SteerDirection> {
        match (self, target) {
            (Self::Down, Self::Down) => None,
            (Self::Down, Self::Left(_)) => Some(SteerDirection::Left),
            (Self::Down, Self::Right(_)) => Some(SteerDirection::Right),
            (Self::Left(_), Self::Down) => Some(SteerDirection::Right),
            (Self::Right(_), Self::Down) => Some(SteerDirection::Left),
            (Self::Right(_x), Self::Left(_y)) => Some(SteerDirection::Left),
            (Self::Left(_x), Self::Right(_y)) => Some(SteerDirection::Right),
            (Self::Left(x), Self::Left(y)) => {
                if x == y {
                    None
                } else if x > y {
                    Some(SteerDirection::Right)
                } else {
                    Some(SteerDirection::Left)
                }
            },
            (Self::Right(x), Self::Right(y)) => {
                if x == y {
                    None
                } else if x > y {
                    Some(SteerDirection::Left)
                } else {
                    Some(SteerDirection::Right)
                }
            },
        }
    }

    pub fn steer_left(&self) -> Self {
        match self {
            Self::Left(x) if x == &SIDES_MAX_INDEX => Self::Left(SIDES_MAX_INDEX),
            Self::Left(x) => Self::Left(x+1),
            Self::Down => Self::Left(0),
            Self::Right(0) => Self::Down,
            Self::Right(x) => Self::Right(x-1),
        }
    }

    pub fn steer_right(&self) -> Self {
        match self {
            Self::Right(x) if x == &SIDES_MAX_INDEX => Self::Right(SIDES_MAX_INDEX),
            Self::Right(x) => Self::Right(x+1),
            Self::Down => Self::Right(0),
            Self::Left(0) => Self::Down,
            Self::Left(x) => Self::Left(x-1),
        }
    }

    pub fn get_standing_position_offset(&self) -> Vec<(f32, f32)> {
        match self {
            Self::Down | Self::Right(0) | Self::Left(0) => vec![(-2.* SCALE_FACTOR, -1.* SCALE_FACTOR), (2.* SCALE_FACTOR, -1.* SCALE_FACTOR)],
            Self::Right(1) | Self::Left(1) => vec![(-2.* SCALE_FACTOR, -2.* SCALE_FACTOR), (2.* SCALE_FACTOR, -2.* SCALE_FACTOR)],
            Self::Right(2) | Self::Left(2) => vec![(-2.* SCALE_FACTOR, -3.* SCALE_FACTOR), (2.* SCALE_FACTOR, -3.* SCALE_FACTOR)],
            Self::Right(3) => vec![(-2.* SCALE_FACTOR, -4.* SCALE_FACTOR), (2.* SCALE_FACTOR, -3.* SCALE_FACTOR)],
            Self::Left(3) => vec![(-2.* SCALE_FACTOR, -3.* SCALE_FACTOR), (2.* SCALE_FACTOR, -4.* SCALE_FACTOR)],
            _ => vec![]
        }
    }

    pub fn get_graphics(&self, game_resources: &GameResources) -> (Rect, bool) {
        match self {
            Self::Down =>(game_resources.down, false),
            Self::Left(x) => (game_resources.sides[*x], true),
            Self::Right(x) => (game_resources.sides[*x], false),
        }
    }

    pub fn get_move_data(&self, speed_modifiers: &SpeedModifiers) -> (f32, f32) {
        match self {
            Self::Down => (0.0, speed_modifiers.down),
            Self::Left(x) => (-TURN_ANGLES[*x], speed_modifiers.side[*x]),
            Self::Right(x) => (TURN_ANGLES[*x], speed_modifiers.side[*x]),
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
pub struct Score {
    value: f32
}

impl Score {
    pub fn increase(&mut self) {
        self.value = self.value + 10.0;
    }
    pub fn decrease(&mut self) {
        self.value = self.value - 10.0;
    }
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
                    keyboard_input,
                    mouse_input,
                    touch_input,
                    update_player.after(keyboard_input),
                    gameover_detection.after(update_player),
                    update_score_text,
                ).in_set(OnUpdate(GameState::Playing))
            );
    }
}


fn keyboard_input(
    keyboard_input: Res<Input<KeyCode>>,
    game_resources: Res<GameResources>,
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
        let (sprite_rect, flip_x) = direction.get_graphics(&game_resources);
        sprite.rect = Some(sprite_rect);
        sprite.flip_x = flip_x;
    }
}

fn steer_to_position(
    position: Vec2,
    game_resources: &GameResources,
    player_transform: &Transform,
    direction: &mut Direction,
    player: &mut Player,
    sprite: &mut Sprite,
) {
    let delta_v = position - player_transform.translation.truncate();
    let angle = delta_v.y.atan2(delta_v.x) + FRAC_PI_2;
    let target_direction = Direction::from_angle(angle);
    let Some(steer_direction) = direction.steering_direction(&target_direction) else {
        return;
    };

    match steer_direction {
        SteerDirection::Left => *direction = direction.steer_left(),
        SteerDirection::Right => *direction = direction.steer_right(),
    };

    player.turn_rate.reset();
    let (sprite_rect, flip_x) = direction.get_graphics(&game_resources);
    sprite.rect = Some(sprite_rect);
    sprite.flip_x = flip_x;
}

fn mouse_input(
    mouse_button_input: Res<Input<MouseButton>>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    game_resources: Res<GameResources>,
    mut player_q: Query<(&Transform, &mut Sprite, &mut Direction, &mut Player), (With<Alive>, Without<Camera>)>
) {
    let Ok((player_transform, mut sprite, mut direction, mut player)) = player_q.get_single_mut() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };
    if !player.turn_rate.finished() {
        return;
    }

    if mouse_button_input.pressed(MouseButton::Left) {
        let Some(mouse_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate()) else
        {
            return;
        };

        steer_to_position(
            mouse_position,
            &game_resources,
            player_transform,
            &mut direction,
            &mut player,
            &mut sprite
        );
    }
}

fn touch_input(
    touches: Res<Touches>,
    window: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
    game_resources: Res<GameResources>,
    mut player_q: Query<(&Transform, &mut Sprite, &mut Direction, &mut Player), (With<Alive>, Without<Camera>)>
) {
    let Ok((player_transform, mut sprite, mut direction, mut player)) = player_q.get_single_mut() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };
    let Ok((camera, camera_transform)) = camera_q.get_single() else {
        return;
    };
    if !player.turn_rate.finished() {
        return;
    }

    let Some(touch) = touches.first_pressed_position() else {
        return;
    };
    let touch_position = vec2(touch.x, window.height() - touch.y);
    let Some(touch_position) = camera
        .viewport_to_world(
            camera_transform,
            touch_position
        )
        .map(|ray| ray.origin.truncate()) else
    {
        return;
    };
    steer_to_position(
        touch_position,
        &game_resources,
        player_transform,
        &mut direction,
        &mut player,
        &mut sprite
    );
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

    let (deg_rad, speed_modifier) = direction.get_move_data(&speed_modifiers);
    let deg_rad = deg_rad - FRAC_PI_2; //0 degrees is pointing down (e.g. [0, -1], not to [1, 0])
    let dx = deg_rad.cos() * speed.0 * speed_modifier;
    let dy = deg_rad.sin() * speed.0 * speed_modifier;

    player_transform.translation.x += dx * dt.as_secs_f32();
    player_transform.translation.y += dy * dt.as_secs_f32();

    let Ok(mut camera_transform) = camera_q.get_single_mut() else {
        return;
    };
    camera_transform.translation.x = player_transform.translation.x;
    camera_transform.translation.y = player_transform.translation.y - PLAYER_CAMERA_OFFSET;
}

fn update_score_text(
    player_q: Query<&Score, With<Player>>,
    mut text_q: Query<&mut Text, With<ScoreText>>,
) {
    let Ok(score) = player_q.get_single() else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };

    text.sections[0].value = format!("Score: {:.0}", score.value);
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
    let text_style = TextStyle {
        font: game_resources.font_handle.clone(),
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
            texture: game_resources.image_handle.clone(),
            transform: Transform::from_xyz(0., 0., PLAYER_Z_INDEX),
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
        Direction::Down,
        Score { value: 0.0 }
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
