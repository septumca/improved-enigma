use std::{f32::consts::{FRAC_PI_2, PI, FRAC_PI_8}};
use bevy::{prelude::*, math::vec2};

use crate::{GameResources, Alive, GameState, collidable::Collidable, despawn, debug::{DebugMarker}, SCALE_FACTOR, uicontrols::{UiControlType, steer_player}, yeti::{YetiState, Yeti}};


const SPEED: f32 = 50.0 * SCALE_FACTOR;
const PLAYER_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
const PLAYER_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -2.0 * SCALE_FACTOR);
pub const FALL_TIMEOUT: f32 = 0.3;
const SIDES_MAX_INDEX: usize = 3;
pub const PLAYER_Z_INDEX: f32 = 2.0;
const PLAYER_CAMERA_OFFSET: f32 = 32.0;
const ROTATION_SPEED: f32 = FRAC_PI_2 * 1.5;
const ROTATION_HINDERANCE: f32 = FRAC_PI_8 / 2.0; //at faster speed the turning is harder, this can be later upgraded to be closer to zero
const SPRITE_ROTATION_TRESHOLD: f32 = FRAC_PI_8 / 2.0;

#[derive(Debug, Component)]
pub struct Direction {
    pub rotation: f32
}

impl Direction {
    pub fn get_graphics_index(&self) -> Option<usize> {
        let angle = self.rotation.abs();
        if angle <= SPRITE_ROTATION_TRESHOLD {
            return None;
        }
        if angle > SPRITE_ROTATION_TRESHOLD && angle <= SPRITE_ROTATION_TRESHOLD * 3.0 {
            return Some(0);
        }
        if angle > SPRITE_ROTATION_TRESHOLD * 3.0 && angle <= SPRITE_ROTATION_TRESHOLD * 5.0 {
            return Some(1);
        }
        if angle > SPRITE_ROTATION_TRESHOLD * 5.0 && angle <= SPRITE_ROTATION_TRESHOLD * 7.0 {
            return Some(2);
        }
        Some(SIDES_MAX_INDEX)
    }

    pub fn is_facing_right(&self) -> bool {
        self.rotation > 0.0 && self.rotation < PI
    }

    pub fn get_graphics(&self, game_resources: &GameResources) -> (Rect, bool) {
        if let Some(side_index) = self.get_graphics_index() {
            return (game_resources.sides[side_index], !self.is_facing_right())
        }
        (game_resources.down, false)
    }

    fn get_rotation_hinderance(&self, speed: &Speed) -> f32 {
        if speed.max_speed == speed.min_speed {
            return ROTATION_HINDERANCE;
        }
        (speed.get_speed(self.rotation) - speed.min_speed) / (speed.max_speed - speed.min_speed) * ROTATION_HINDERANCE
    }

    pub fn steer_left(&mut self, delta: f32, speed: &Speed) {
        let rot_hinderance = self.get_rotation_hinderance(speed);
        self.rotation = (self.rotation - (ROTATION_SPEED - rot_hinderance) * delta).max(-FRAC_PI_2);
    }

    pub fn steer_right(&mut self, delta: f32, speed: &Speed) {
        let rot_hinderance = self.get_rotation_hinderance(speed);
        self.rotation = (self.rotation + (ROTATION_SPEED - rot_hinderance) * delta).min(FRAC_PI_2);
    }

    pub fn get_standing_position_offset(&self) -> Vec<(f32, f32)> {
        let side_index = self.get_graphics_index();
        match side_index {
            None | Some(0) => vec![(-2.* SCALE_FACTOR, -1.* SCALE_FACTOR), (2.* SCALE_FACTOR, -1.* SCALE_FACTOR)],
            Some(1) => vec![(-2.* SCALE_FACTOR, -2.* SCALE_FACTOR), (2.* SCALE_FACTOR, -2.* SCALE_FACTOR)],
            Some(2) | Some(3) => vec![(-2.* SCALE_FACTOR, -3.* SCALE_FACTOR), (2.* SCALE_FACTOR, -3.* SCALE_FACTOR)],
            _ => vec![]
        }
    }

    pub fn get_skis_transform_y(&self) -> (f32, f32) {
        match self.get_graphics_index() {
            None | Some(0) => (-3.0 * SCALE_FACTOR, -3.0 * SCALE_FACTOR),
            Some(_) => {
                let lower = -3.0 * SCALE_FACTOR;
                let upper = -2.0 * SCALE_FACTOR;
                if self.is_facing_right() {
                    (lower, upper)
                } else {
                    (upper, lower)
                }
            }
        }
    }
}


#[derive(Component)]
pub struct Speed {
    pub max_speed: f32,
    min_speed: f32,
}

impl Speed {
    pub fn new(max_speed: f32, min_speed_ratio: f32) -> Self {
        Speed {
            max_speed,
            min_speed: max_speed * min_speed_ratio.min(1.0),
        }
    }

    pub fn get_speed(&self, rotation: f32) -> f32 {
        (self.max_speed - self.min_speed) * rotation.cos().abs() + self.min_speed
    }
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
pub struct Player;


#[derive(Component)]
pub struct Slowdown(pub Timer);


#[derive(Component)]
struct ScoreText;

#[derive(Component)]
struct GameOverText;

#[derive(Component)]
pub struct LeftSki;

#[derive(Component)]
pub struct RightSki;


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
                    update_player.after(keyboard_input),
                    gameover_detection.after(update_player),
                    update_score_text,
                ).in_set(OnUpdate(GameState::Playing))
            );
    }
}


fn keyboard_input(
    timer: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    game_resources: Res<GameResources>,
    mut player_q: Query<(&mut Sprite, &mut Direction, &Speed), (With<Player>, With<Alive>, Without<LeftSki>, Without<RightSki>)>,
    mut left_ski: Query<&mut Transform, (With<LeftSki>, Without<RightSki>)>,
    mut right_ski: Query<&mut Transform, (With<RightSki>, Without<LeftSki>)>
) {
    let Ok((mut sprite, mut direction, speed)) = player_q.get_single_mut() else {
        return;
    };
    let Ok(mut lski_transform) = left_ski.get_single_mut() else {
        return;
    };
    let Ok(mut rski_transform) = right_ski.get_single_mut() else {
        return;
    };

    let mut control_type = None;
    if keyboard_input.pressed(KeyCode::A) {
        control_type = Some(UiControlType::Left);
    }
    if keyboard_input.pressed(KeyCode::D) {
        control_type = Some(UiControlType::Right);
    }
    let Some(control_type) = control_type else {
        return;
    };
    let dt = timer.delta_seconds();

    steer_player(
        dt,
        &control_type,
        &game_resources,
        &mut sprite,
        &mut direction,
        speed,
        &mut lski_transform,
        &mut rski_transform
    );


}

pub fn update_player(
    timer: Res<Time>,
    mut player_q: Query<(&mut Transform, &Speed, &Direction), (With<Player>, Without<Camera>)>,
    mut camera_q: Query<&mut Transform, With<Camera>>
) {
    let Ok((
        mut player_transform,
        speed,
        direction,
    )) = player_q.get_single_mut() else {
        return;
    };
    let dt = timer.delta();

    let act_speed = speed.get_speed(direction.rotation);
    let act_rotation = direction.rotation - FRAC_PI_2; //0 degrees is pointing down (e.g. [0, -1], not to [1, 0])
    let dx = act_rotation.cos() * act_speed;
    let dy = act_rotation.sin() * act_speed;

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
    mut player_q: Query<Option<&mut Slowdown>, (With<Player>, Without<Alive>)>,
    yeti_q: Query<&YetiState, (With<Yeti>, Without<Player>)>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    let mut is_game_over = false;
    if let Ok(slowdown) = player_q.get_single_mut() {
        if let Some(mut slowdown) = slowdown {
            if slowdown.0.tick(time.delta()).just_finished() {
                is_game_over = true;
            }
        } else {
            if let Ok(yeti_state) = yeti_q.get_single() {
                is_game_over = *yeti_state == YetiState::Catched;
            }
        }
    };

    if is_game_over {
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
        Player,
        Alive,
        Collidable::new(
            0., 0.,
            PLAYER_COLLIDABLE_DIMENSIONS.0, PLAYER_COLLIDABLE_DIMENSIONS.1,
            PLAYER_COLLIDABLE_OFFSETS.0, PLAYER_COLLIDABLE_OFFSETS.1
        ),
        Speed::new(SPEED, 0.4),
        Direction {
            rotation: 0.0,
        },
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

        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(1.0 * SCALE_FACTOR, 6.0 * SCALE_FACTOR)),
                    ..default()
                },
                transform: Transform::from_xyz(-1.5 * SCALE_FACTOR, -3.0 * SCALE_FACTOR, -0.5),
                ..default()
            },
            LeftSki
        ));

        parent.spawn((
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(1.0 * SCALE_FACTOR, 6.0 * SCALE_FACTOR)),
                    ..default()
                },
                transform: Transform::from_xyz(1.5 * SCALE_FACTOR, -3.0 * SCALE_FACTOR, -0.5),
                ..default()
            },
            RightSki
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
