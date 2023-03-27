use bevy::{prelude::*};

use crate::{GameState, despawn, GameResources, player::{Player, Direction, RightSki, LeftSki, Speed}, Alive, NORMAL_BUTTON};

#[derive(Component)]
struct UiControls;

#[derive(Component)]
pub enum UiControlType {
    Left,
    Right,
}

pub struct UiControlsPlugin;

impl Plugin for UiControlsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(setup_uicontrols.in_schedule(OnEnter(GameState::Playing)))
            .add_system(despawn::<UiControls>.in_schedule(OnExit(GameState::Playing)))
            .add_system(controls_interaction.in_set(OnUpdate(GameState::Playing)));
    }
}

pub fn steer_player(
    delta: f32,
    control_type: &UiControlType,
    game_resources: &GameResources,
    sprite: &mut Sprite,
    direction: &mut Direction,
    speed: &Speed,
    lski_transform: &mut Transform,
    rski_transform: &mut Transform,
) {
    match control_type {
        UiControlType::Left => direction.steer_left(delta, speed),
        UiControlType::Right => direction.steer_right(delta, speed),
    };

    let (lski_y, rski_y) = direction.get_skis_transform_y();
    lski_transform.translation.y = lski_y;
    rski_transform.translation.y = rski_y;
    lski_transform.rotation = Quat::from_rotation_z(direction.rotation);
    rski_transform.rotation = Quat::from_rotation_z(direction.rotation);
    let (sprite_rect, flip_x) = direction.get_graphics(&game_resources);
    sprite.rect = Some(sprite_rect);
    sprite.flip_x = flip_x;
}

fn controls_interaction(
    timer: Res<Time>,
    mut player_q: Query<(&mut Sprite, &mut Direction, &Speed), (With<Player>, With<Alive>)>,
    mut left_ski: Query<&mut Transform, (With<LeftSki>, Without<RightSki>)>,
    mut right_ski: Query<&mut Transform, (With<RightSki>, Without<LeftSki>)>,
    game_resources: Res<GameResources>,
    interaction_query: Query<
        (&Interaction, &UiControlType),
        (With<Button>, With<UiControls>),
    >,
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

    for (interaction, uicontrol_type) in &interaction_query {
        match *interaction {
            Interaction::Clicked | Interaction::Hovered => {
                let dt = timer.delta_seconds();
                steer_player(
                    dt,
                    uicontrol_type,
                    &game_resources,
                    &mut sprite,
                    &mut direction,
                    speed,
                    &mut lski_transform,
                    &mut rski_transform
                );
            },
            _ => {}
        }
    }
}

fn setup_uicontrols(
    mut commands: Commands,
    window: Query<&Window>,
    game_resources: Res<GameResources>,
) {
    let text_style = TextStyle {
        font: game_resources.font_handle.clone(),
        font_size: 32.0,
        color: Color::BLACK,
    };
    let Ok(window) = window.get_single() else {
        return;
    };
    let size = window.width() / 4.0;
    commands.spawn((
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(size), Val::Px(size)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                ..default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..default()
        },
        UiControls,
        UiControlType::Right
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            ">",
            text_style.clone(),
        ));
    });

    commands.spawn((
        ButtonBundle {
            style: Style {
                size: Size::new(Val::Px(size), Val::Px(size)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    left: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..default()
                },
                ..default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..default()
        },
        UiControls,
        UiControlType::Left
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "<",
            text_style.clone(),
        ));
    });
}