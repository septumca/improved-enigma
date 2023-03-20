use std::{f32::consts::{FRAC_PI_2, FRAC_PI_8, FRAC_PI_4}};
use bevy::{prelude::*, math::vec2};

use crate::{GameResources, Alive, GameState, collidable::Collidable, cleanup};


const SPEED: f32 = 50.0;
const TURN_RATE: f32 = 0.15;
const SCORE_RATIO: f32 = 0.05;
const PLAYER_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.0, 3.5);
pub const FALL_TIMEOUT: f32 = 0.3;
const SIDE_STOP: f32 = 1.2;
const SIDES_MAX_INDEX: usize = 3;



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
pub struct Player {
    turn_rate: Timer
}

impl Player {
    pub fn new(turn_rate_seconds: f32) -> Self {
        Self { turn_rate: Timer::from_seconds(turn_rate_seconds, TimerMode::Once) }
    }
}

#[derive(Component)]
struct Trail;

#[derive(Component)]
pub struct Slowdown(pub Timer);


#[derive(Resource)]
struct Score {
    value: f32
}

impl Score {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }
}

#[derive(Component)]
struct ScoreText {}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Score::new())
            .add_system(setup.in_schedule(OnEnter(GameState::Playing)))
            .add_systems(
                (
                    input,
                    update_player.after(input),
                    movement_slowdown.after(input),
                    leave_trail.after(update_player),
                    update_score.after(update_player),
                    gameover_detection.after(update_player),
                    cleanup::<Trail>,
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
                color: Color::rgb(0.8, 0.8, 0.9),
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

fn setup(
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
        font_size: 4.0,
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
