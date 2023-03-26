use bevy::{prelude::*};

use crate::{GameState, despawn, GameResources, player::{Player, Direction}, Alive, NORMAL_BUTTON};

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
    control_type: &UiControlType,
    game_resources: &GameResources,
    sprite: &mut Sprite,
    direction: &mut Direction,
    player: &mut Player
) {
    match control_type {
        UiControlType::Left => *direction = direction.steer_left(),
        UiControlType::Right => *direction = direction.steer_right(),
    }

    player.turn_rate.reset();
    let (sprite_rect, flip_x) = direction.get_graphics(&game_resources);
    sprite.rect = Some(sprite_rect);
    sprite.flip_x = flip_x;
}

fn controls_interaction(
    mut player_q: Query<(&mut Sprite, &mut Direction, &mut Player), With<Alive>>,
    game_resources: Res<GameResources>,
    interaction_query: Query<
        (&Interaction, &UiControlType),
        (With<Button>, With<UiControls>),
    >,
) {
    let Ok((mut sprite, mut direction, mut player)) = player_q.get_single_mut() else {
        return;
    };
    if !player.can_turn() {
        return;
    }
    for (interaction, uicontrol_type) in &interaction_query {
        match *interaction {
            Interaction::Clicked | Interaction::Hovered => {
                steer_player(
                    uicontrol_type,
                    &game_resources,
                    &mut sprite,
                    &mut direction,
                    &mut player
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
        font_size: 24.0,
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