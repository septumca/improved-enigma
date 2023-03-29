use bevy::{prelude::*};

use crate::{GameState, despawn, GameResources, NORMAL_BUTTON};

#[derive(Component)]
struct GameOverElement;

#[derive(Component)]
enum GameOverControl {
    Restart,
    MainMenu
}

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(setup_gameover.in_schedule(OnEnter(GameState::GameOver)))
            .add_system(despawn::<GameOverElement>.in_schedule(OnExit(GameState::GameOver)))
            .add_system(controls_interaction.in_set(OnUpdate(GameState::GameOver)));
    }
}

fn controls_interaction(
    interaction_query: Query<
        (&Interaction, &GameOverControl),
        (Changed<Interaction>, With<Button>, With<GameOverElement>),
    >,
    mut app_state: ResMut<NextState<GameState>>,
) {
    for (interaction, control) in &interaction_query {
        match *interaction {
            Interaction::Clicked => {
                match control {
                    GameOverControl::MainMenu => {
                        app_state.set(GameState::MainMenu);
                    },
                    GameOverControl::Restart => {
                        app_state.set(GameState::Playing);
                    }
                }
            },
            _ => {}
        }
    }
}

fn setup_gameover(
    mut commands: Commands,
    game_resources: Res<GameResources>,
) {
    let text_style = TextStyle {
        font: game_resources.font_handle.clone(),
        font_size: 24.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Center;

    commands.spawn((
        TextBundle::from_section(
            "Game Over",
            text_style.clone(),
        )
        .with_text_alignment(text_alignment)
        .with_style(Style {
            position_type: PositionType::Absolute,
            position: UiRect {
                top: Val::Px(200.0),
                ..default()
            },
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            ..default()
        }),
        GameOverElement,
    ));

    commands.spawn((
        ButtonBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(300.0),
                    ..default()
                },
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },
                padding: UiRect {
                    left: Val::Px(12.0),
                    right: Val::Px(12.0),
                    top: Val::Px(8.0),
                    bottom: Val::Px(8.0)
                },
                ..default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..default()
        },
        GameOverElement,
        GameOverControl::Restart,
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Restart",
            text_style.clone(),
        ));
    });

    commands.spawn((
        ButtonBundle {
            style: Style {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                position: UiRect {
                    top: Val::Px(350.0),
                    ..default()
                },
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },
                padding: UiRect {
                    left: Val::Px(12.0),
                    right: Val::Px(12.0),
                    top: Val::Px(8.0),
                    bottom: Val::Px(8.0)
                },
                ..default()
            },
            background_color: NORMAL_BUTTON.into(),
            ..default()
        },
        GameOverElement,
        GameOverControl::MainMenu,
    ))
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "Main Menu",
            text_style.clone(),
        ));
    });
}