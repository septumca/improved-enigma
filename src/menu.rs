use std::{fmt};
use bevy::{prelude::*};

use crate::{GameState, GameResources, despawn};


pub struct MenuPlugin;

#[derive(Clone)]
enum MainMenuItem {
    Play,
}

impl fmt::Display for MainMenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MainMenuItem::Play => write!(f, "Play"),
        }
    }
}


#[derive(Component)]
struct MenuText;

#[derive(Component)]
struct MainMenuItems(Vec<MainMenuItem>);

#[derive(Component)]
struct SelectedItemIndex(usize);

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(setup.in_schedule(OnEnter(GameState::MainMenu)))
            .add_systems(
                (
                    input,
                ).in_set(OnUpdate(GameState::MainMenu))
            )
            .add_system(despawn::<MenuText>.in_schedule(OnExit(GameState::MainMenu)));
    }
}

fn setup(
    mut commands: Commands,
    game_resources: Res<GameResources>,
) {
    let Some(font_handle) = &game_resources.font_handle else {
        return;
    };

    let text_style = TextStyle {
        font: font_handle.clone(),
        font_size: 36.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Center;

    let items = vec![MainMenuItem::Play];
    commands.spawn(MainMenuItems(items.clone()));
    commands.spawn(SelectedItemIndex(0));

    for (index, item ) in items.iter().enumerate() {
        commands.spawn((
            TextBundle::from_section(
                item.to_string(),
                text_style.clone(),
            )
            .with_text_alignment(text_alignment)
            .with_style(Style {
                position: UiRect {
                    top: Val::Px(40.0 * index as f32 * 120.0),
                    ..default()
                },
                margin: UiRect {
                    left: Val::Auto,
                    right: Val::Auto,
                    ..default()
                },
                ..default()
            }),
            MenuText
        ));
    }
}

fn input(
    mut keyboard_input: ResMut<Input<KeyCode>>,
    items_q: Query<&MainMenuItems>,
    mut index_q: Query<&mut SelectedItemIndex>,
    mut app_state: ResMut<NextState<GameState>>,
) {
    let Ok(items) = items_q.get_single() else {
        return;
    };
    let Ok(mut index) = index_q.get_single_mut() else {
        return;
    };

    if keyboard_input.just_pressed(KeyCode::Up) {
        index.0 = prev_item(index.0, items.0.len());
    }
    if keyboard_input.just_pressed(KeyCode::Down) {
        index.0 = next_item(index.0, items.0.len());
    }
    if keyboard_input.just_pressed(KeyCode::Return) {
        match items.0[index.0] {
            MainMenuItem::Play => {
                app_state.set(GameState::Playing);
                keyboard_input.reset(KeyCode::Return);
            },
        }
    }
}

fn prev_item(selected_item_index: usize, length: usize) -> usize {
    if selected_item_index == 0 {
        length - 1
    } else {
        selected_item_index - 1
    }
}

pub fn next_item(selected_item_index: usize, length: usize) -> usize {
    if selected_item_index + 1 >= length {
        0
    } else {
        selected_item_index + 1
    }
}
