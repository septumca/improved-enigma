use bevy::{prelude::*};

use crate::{GameState, despawn, GameResources};

#[derive(Component)]
struct GameOverText;

pub struct GameOverPlugin;

impl Plugin for GameOverPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(setup_gameover.in_schedule(OnEnter(GameState::GameOver)))
            .add_system(despawn::<GameOverText>.in_schedule(OnExit(GameState::GameOver)))
            .add_system(input_gameover.in_set(OnUpdate(GameState::GameOver)));
    }
}

fn input_gameover(
    keyboard_input: Res<Input<KeyCode>>,
    touches: Res<Touches>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::R) || touches.any_just_pressed() {
        app_state.set(GameState::Playing);
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
            "Game Over\n\n\n\n\nPress R\nor touch the screen\nto try again!",
            text_style.clone(),
        )
        .with_text_alignment(text_alignment)
        .with_style(Style {
            align_self: AlignSelf::Center,
            margin: UiRect {
                left: Val::Auto,
                right: Val::Auto,
                ..default()
            },
            ..default()
        }),
        GameOverText
    ));
}