use std::{f32::consts::{FRAC_PI_2, PI, FRAC_PI_8}};
use bevy::{prelude::*, math::vec2};

use crate::{
    GameResources,
    Alive,
    GameState,
    collidable::{Collidable, CollidableMovable},
    despawn,
    debug::{DebugMarker},
    SCALE_FACTOR,
    uicontrols::{UiControlType, self}, stuneffect::{Stun, StunEffect}, camera::CameraFocus, animation::AnimateRotation
};


const SPEED: f32 = 50.0 * SCALE_FACTOR;
const ACCELERATION: f32 = 20.0 * SCALE_FACTOR;
const PLAYER_COLLIDABLE_DIMENSIONS: (f32, f32) = (2.0 * SCALE_FACTOR, 2.0 * SCALE_FACTOR);
const PLAYER_COLLIDABLE_OFFSETS: (f32, f32) = (0.0 * SCALE_FACTOR, -2.0 * SCALE_FACTOR);
pub const FALL_TIMEOUT: f32 = 0.3;
const SIDES_MAX_INDEX: usize = 3;
pub const PLAYER_Z_INDEX: f32 = 2.0;
pub const PLAYER_CAMERA_OFFSET: f32 = 32.0;
const ROTATION_SPEED: f32 = FRAC_PI_2 * 1.5;
const ROTATION_HINDERANCE: f32 = FRAC_PI_8 / 2.0; //at faster speed the turning is harder, this can be later upgraded to be closer to zero
const SPRITE_ROTATION_TRESHOLD: f32 = FRAC_PI_8 / 2.0;
const ROTATION_HINDERANCE_SLOPE: f32 = 15.0;


fn get_graphics_index(rotation: f32) -> Option<usize> {
    let angle = rotation.abs();

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

fn is_facing_right(rotation: f32) -> bool {
    rotation > 0.0 && rotation < PI
}

pub fn get_graphics(rotation: f32, game_resources: &GameResources) -> (Rect, bool) {
    if let Some(side_index) = get_graphics_index(rotation) {
        return (game_resources.sides[side_index], !is_facing_right(rotation))
    }
    (game_resources.down, false)
}

pub fn get_standing_position_offset(rotation: f32) -> Vec<(f32, f32)> {
    let side_index = get_graphics_index(rotation);
    match side_index {
        None | Some(0) => vec![(-2.* SCALE_FACTOR, -1.* SCALE_FACTOR), (2.* SCALE_FACTOR, -1.* SCALE_FACTOR)],
        Some(1) => vec![(-2.* SCALE_FACTOR, -2.* SCALE_FACTOR), (2.* SCALE_FACTOR, -2.* SCALE_FACTOR)],
        Some(2) | Some(3) => vec![(-2.* SCALE_FACTOR, -3.* SCALE_FACTOR), (2.* SCALE_FACTOR, -3.* SCALE_FACTOR)],
        _ => vec![]
    }
}

fn get_skis_transform_y(rotation: f32) -> (f32, f32) {
    match get_graphics_index(rotation) {
        None | Some(0) => (-3.0 * SCALE_FACTOR, -3.0 * SCALE_FACTOR),
        Some(_) => {
            let lower = -3.0 * SCALE_FACTOR;
            let upper = -2.0 * SCALE_FACTOR;
            if is_facing_right(rotation) {
                (lower, upper)
            } else {
                (upper, lower)
            }
        }
    }
}

fn get_rotation_hinderance(speed: f32) -> f32 {
    (speed / (speed + ROTATION_HINDERANCE_SLOPE)) * ROTATION_HINDERANCE
}

#[derive(Debug, Component)]
pub struct Rotation(pub f32);

#[derive(Component)]
pub struct Velocity(pub Vec2);


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
    pub control_type: Option<UiControlType>,
    speed: f32,
    max_speed: f32,
    min_speed: f32,
    max_acceleration: f32,
}

impl Player {
    pub fn new(max_speed: f32, min_speed_ratio: f32, max_acceleration: f32) -> Self {
        Self {
            control_type: None,
            speed: 0.0,
            max_speed,
            min_speed: max_speed * min_speed_ratio.min(0.95),
            max_acceleration,
        }
    }

    pub fn get_speed(&self, rotation: f32) -> f32 {
        (self.max_speed - self.min_speed) * rotation.cos().abs() + self.min_speed
    }

    pub fn get_acceleration(&self, rotation: f32) -> f32 {
        self.max_acceleration * rotation.cos().abs()
    }

    pub fn deaccelerate(&mut self, rotation: f32) {
        self.speed = self.get_speed(rotation);
    }

    pub fn accelerate(&mut self, rotation: f32, delta: f32) {
        let act_acceleration = self.get_acceleration(rotation) * delta;
        self.speed = (self.speed + act_acceleration).min(self.max_speed)
    }
}

#[derive(Component)]
pub struct CompletedRace;

#[derive(Component)]
pub struct Falldown;

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

#[derive(Component)]
pub struct Catched;
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
                    update_player.after(uicontrols::player_input),
                    update_graphics.after(update_player),
                    update_movables.after(update_player),
                    update_slowdown,
                    update_fallover.after(update_player),
                    gameover_detection.after(update_player),
                    update_ski_rotation.after(update_player),
                    update_score,
                ).in_set(OnUpdate(GameState::Playing))
            );
    }
}

pub fn update_player(
    timer: Res<Time>,
    mut player_q: Query<(&mut Velocity, &mut Rotation, &mut Player), Without<Stun>>,
) {
    let Ok((
        mut velocity,
        mut rotation,
        mut player
    )) = player_q.get_single_mut() else {
        return;
    };

    let dt = timer.delta();
    if let Some(control_type) = &player.control_type {
        let rot_hinderance = get_rotation_hinderance(player.speed);
        let rot_delta = match control_type {
            UiControlType::Left => -(ROTATION_SPEED - rot_hinderance),
            UiControlType::Right => ROTATION_SPEED - rot_hinderance,
        };
        rotation.0 = (rotation.0 + rot_delta * dt.as_secs_f32())
            .max(-FRAC_PI_2)
            .min(FRAC_PI_2);
        player.deaccelerate(rotation.0);
    } else {
        player.accelerate(rotation.0, dt.as_secs_f32());
    }

    player.control_type = None;
    let act_rotation = rotation.0 - FRAC_PI_2; //0 degrees is pointing down (e.g. [0, -1], not to [1, 0])
    velocity.0 = vec2(act_rotation.cos() * player.speed, act_rotation.sin() * player.speed);
}

pub fn update_movables(
    timer: Res<Time>,
    mut movables_q: Query<(&mut Transform, &Velocity), Without<Stun>>,
) {
    let dt = timer.delta();
    for (mut transform, vel) in movables_q.iter_mut() {
        transform.translation.x += vel.0.x * dt.as_secs_f32();
        transform.translation.y += vel.0.y * dt.as_secs_f32();
    }
}

fn update_ski_rotation(
    player_q: Query<(&Rotation, &Children), (Without<LeftSki>, Without<RightSki>, With<Player>)>,
    mut left_ski: Query<&mut Transform, (With<LeftSki>, Without<RightSki>)>,
    mut right_ski: Query<&mut Transform, (With<RightSki>, Without<LeftSki>)>
) {
    let Ok((rotation, children)) = player_q.get_single() else {
        return;
    };
    let (lski_y, rski_y) = get_skis_transform_y(rotation.0);
    for &ch in children {
        if let Ok(mut lski_transform) = left_ski.get_mut(ch) {
            lski_transform.translation.y = lski_y;
            lski_transform.rotation = Quat::from_rotation_z(rotation.0);
        }
        if let Ok(mut rski_transform) = right_ski.get_mut(ch) {
            rski_transform.translation.y = rski_y;
            rski_transform.rotation = Quat::from_rotation_z(rotation.0);
        }
    }
}

fn update_graphics(
    game_resources: Res<GameResources>,
    mut player_q: Query<(&mut Sprite, &Rotation), (With<Player>, With<Alive>, Without<Stun>, Without<Falldown>)>,
) {
    let Ok((
        mut sprite,
        rotation,
    )) = player_q.get_single_mut() else {
        return;
    };

    let (sprite_rect, flip_x) = get_graphics(rotation.0, &game_resources);
    sprite.rect = Some(sprite_rect);
    sprite.flip_x = flip_x;
}

fn update_score(
    time: Res<Time>,
    mut player_q: Query<&mut Score, With<Player>>,
    mut text_q: Query<&mut Text, With<ScoreText>>,
) {
    let Ok(mut score) = player_q.get_single_mut() else {
        return;
    };
    let Ok(mut text) = text_q.get_single_mut() else {
        return;
    };
    score.value += time.delta_seconds();
    text.sections[0].value = format!("Time: {:.0}", score.value);
}

fn update_slowdown(
    time: Res<Time>,
    mut commands: Commands,
    mut player_q: Query<(Entity, &mut Slowdown, &mut Player)>,
) {
    let Ok((entity, mut slowdown, mut player)) = player_q.get_single_mut() else {
        return;
    };
    if !slowdown.0.tick(time.delta()).finished() {
        return;
    }

    player.speed = 0.0;
    commands.entity(entity).remove::<Slowdown>();
}

fn update_fallover(
    game_resources: Res<GameResources>,
    mut stopped: RemovedComponents<Slowdown>,
    mut commands: Commands,
    mut player_q: Query<(Entity, &mut Player), With<Falldown>>,
) {
    for stopped in stopped.iter() {
        let Ok((entity, mut player)) = player_q.get_mut(stopped) else {
            continue;
        };
        player.speed = 0.0;
        let stun_child = commands.spawn((
            SpriteBundle {
                sprite: Sprite {
                    custom_size: Some(vec2(game_resources.sprite_size, game_resources.sprite_size)),
                    rect: Some(game_resources.stun),
                    ..default()
                },
                texture: game_resources.image_handle.clone(),
                transform: Transform::from_xyz(0.0, 4.0 * SCALE_FACTOR, 0.5),
                ..default()
            },
            AnimateRotation {
                angular_vel: PI
            },
            StunEffect
        )).id();
        commands.entity(entity).remove::<Falldown>();
        commands.entity(entity).push_children(&[stun_child]);
        commands.entity(entity).insert(Stun(Timer::from_seconds(0.5, TimerMode::Once)));
    }


}

fn gameover_detection(
    player_q: Query<Entity, (With<Player>, Without<Slowdown>, Or<(With<Catched>, With<CompletedRace>)>)>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    if let Ok(_) = player_q.get_single() {
        app_state.set(GameState::GameOver);
    }
}

pub fn setup(
    mut commands: Commands,
    game_resources: Res<GameResources>,
    mut camera_q: Query<&mut Transform, With<Camera>>,
) {
    let text_style = TextStyle {
        font: game_resources.font_handle.clone(),
        font_size: 24.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Right;

    let Ok(mut camera_transform) = camera_q.get_single_mut() else {
        return;
    };
    camera_transform.translation.x = 0.0;
    camera_transform.translation.y = -PLAYER_CAMERA_OFFSET;

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
        CameraFocus,
        Player::new(SPEED, 0.4, ACCELERATION),
        Alive,
        Velocity(Vec2::ZERO),
        CollidableMovable,
        Collidable::new(
            0., 0.,
            PLAYER_COLLIDABLE_DIMENSIONS.0, PLAYER_COLLIDABLE_DIMENSIONS.1,
            PLAYER_COLLIDABLE_OFFSETS.0, PLAYER_COLLIDABLE_OFFSETS.1
        ),
        Rotation(0.0),
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
            "Time: 0.0",
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
