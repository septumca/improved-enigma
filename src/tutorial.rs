use bevy::{prelude::*};

use crate::{GameResources, despawn, GameState, is_running_on_desktop};

#[derive(Component)]
pub struct TutorialText;

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(setup.in_schedule(OnEnter(GameState::Playing)))
            .add_system(despawn::<TutorialText>.in_schedule(OnExit(GameState::Playing)));
    }
}

fn setup(
    mut commands: Commands,
    game_resources: Res<GameResources>,
) {
    let text_style = TextStyle {
        font: game_resources.font_handle.clone(),
        font_size: 30.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Center;
    let text = if !is_running_on_desktop() {
        "Press buttons\nto turn"
    } else {
        "Press A and D\nto turn"
    };
    commands.spawn((
        Text2dBundle {
            text: Text::from_section(text, text_style.clone()).with_alignment(text_alignment),
            transform: Transform::from_xyz(0., -250., 0.5),
            ..default()
        },
        TutorialText,
    ));
}