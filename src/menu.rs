use std::{fmt};
use bevy::{prelude::*};

use crate::{GameState, GameResources, despawn, NORMAL_BUTTON};


pub struct MenuPlugin;

#[derive(Clone, Component)]
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
struct Menu;


impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system(setup.in_schedule(OnEnter(GameState::MainMenu)))
            .add_systems(
                (
                    controls_interaction,
                ).in_set(OnUpdate(GameState::MainMenu))
            )
            .add_system(despawn::<Menu>.in_schedule(OnExit(GameState::MainMenu)));
    }
}

fn controls_interaction(
    interaction_query: Query<
        (&Interaction, &MainMenuItem),
        (Changed<Interaction>, With<Button>),
    >,
    mut app_state: ResMut<NextState<GameState>>,
) {
    for (interaction, menu_item) in &interaction_query {
        match *interaction {
            Interaction::Clicked => {
                match menu_item {
                    MainMenuItem::Play => {
                        app_state.set(GameState::Playing);
                    }
                }
            },
            _ => {}
        }
    }
}

fn setup(
    mut commands: Commands,
    game_resources: Res<GameResources>,
) {
    let text_style = TextStyle {
        font: game_resources.font_handle.clone(),
        font_size: 36.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Center;

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size::all(Val::Percent(100.)),
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            Menu
        ))
        .with_children(|builder| {
            builder.spawn((
                ButtonBundle {
                    style: Style {
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        position_type: PositionType::Absolute,
                        position: UiRect {
                            top: Val::Px(100.0),
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
                MainMenuItem::Play,
            ))
            .with_children(|parent| {
                parent.spawn(TextBundle::from_section(
                    "Play",
                    text_style.clone(),
                ));
            });

            builder.spawn((
                TextBundle::from_section(
                    format!("Credits: TODO"),
                    TextStyle {
                        font: game_resources.font_handle.clone(),
                        font_size: 24.0,
                        color: Color::BLACK,
                    },
                )
                .with_text_alignment(text_alignment)
                .with_style(Style {
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
            ));

            builder.spawn((
                TextBundle::from_section(
                    "Press M to toggle music",
                    TextStyle {
                        font: game_resources.font_handle.clone(),
                        font_size: 24.0,
                        color: Color::BLACK,
                    },
                )
                .with_text_alignment(text_alignment)
                .with_style(Style {
                    position: UiRect {
                        top: Val::Px(300.0),
                        ..default()
                    },
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    ..default()
                }),
            ));
        });
}
