use bevy::{prelude::*};

use crate::{GameState, GameResources, despawn, SELECTED_BUTTON, music::MusicResource, uicontrols::ControlScheme, NORMAL_BUTTON, level_generator::LevelGeneratorSettings};


pub struct MenuPlugin;

#[derive(Clone, Component)]
enum MainMenuItem {
    Play,
    MusicOn,
    MusicOff,
    ControlMobile,
    ControlDesktop,
}


struct MenuItemSelected(MainMenuItem);

#[derive(Component)]
struct Menu;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<MenuItemSelected>()
            .add_system(setup.in_schedule(OnEnter(GameState::MainMenu)))
            .add_systems(
                (
                    controls_interaction,
                    reset_buttons.after(controls_interaction),
                ).in_set(OnUpdate(GameState::MainMenu))
            )
            .add_system(despawn::<Menu>.in_schedule(OnExit(GameState::MainMenu)));
    }
}

fn reset_buttons(
    mut button_q: Query<(&mut BackgroundColor, &MainMenuItem), With<Button>>,
    mut ev_menuitemselected: EventReader<MenuItemSelected>,
) {
    for item in ev_menuitemselected.iter() {
        for (mut color, menu_item) in button_q.iter_mut() {
            match (&item.0, menu_item) {
                (MainMenuItem::ControlDesktop, MainMenuItem::ControlMobile) |
                (MainMenuItem::ControlMobile, MainMenuItem::ControlDesktop) |
                (MainMenuItem::MusicOff, MainMenuItem::MusicOn) |
                (MainMenuItem::MusicOn, MainMenuItem::MusicOff) => {
                    *color = get_button_color(false);
                },
                (MainMenuItem::ControlDesktop, MainMenuItem::ControlDesktop) |
                (MainMenuItem::ControlMobile, MainMenuItem::ControlMobile) |
                (MainMenuItem::MusicOff, MainMenuItem::MusicOff) |
                (MainMenuItem::MusicOn, MainMenuItem::MusicOn) => {
                    *color = get_button_color(true);
                },
                _ => {}
            }
        }
    }
}

fn controls_interaction(
    interaction_query: Query<
        (&Interaction, &MainMenuItem),
        (Changed<Interaction>, With<Button>),
    >,
    mut commands: Commands,
    mut app_state: ResMut<NextState<GameState>>,
    mut music_resource: ResMut<MusicResource>,
    mut control_scheme: ResMut<ControlScheme>,
    mut ev_menuitemselected: EventWriter<MenuItemSelected>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    for (interaction, menu_item) in &interaction_query {
        match *interaction {
            Interaction::Clicked => {
                match menu_item {
                    MainMenuItem::Play => {
                        commands.insert_resource(LevelGeneratorSettings {
                            tile_size: 120.0,
                            displacement: 30.0,
                            start_offset_y: -350.0,
                            width: 50,
                            height: 400,
                            starting_difficulty: 0.3
                        });
                        app_state.set(GameState::Playing);
                    },
                    MainMenuItem::MusicOn => {
                        if let Some(sink) = audio_sinks.get(&music_resource.controller) {
                            sink.play();
                            music_resource.playing = true;
                        }
                    },
                    MainMenuItem::MusicOff => {
                        if let Some(sink) = audio_sinks.get(&music_resource.controller) {
                            sink.pause();
                            music_resource.playing = false;
                        }
                    },
                    MainMenuItem::ControlDesktop => {
                        control_scheme.set_desktop();
                    },
                    MainMenuItem::ControlMobile => {
                        control_scheme.set_mobile();
                    }
                };
                ev_menuitemselected.send(MenuItemSelected(menu_item.clone()));
            },
            _ => {}
        }
    }
}

fn get_button_color(toggle: bool) -> BackgroundColor {
    if toggle {
        SELECTED_BUTTON.into()
    } else {
        NORMAL_BUTTON.into()
    }
}

fn setup(
    window: Query<&Window>,
    mut commands: Commands,
    game_resources: Res<GameResources>,
    music_resource: Res<MusicResource>,
    control_scheme: Res<ControlScheme>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let text_style = TextStyle {
        font: game_resources.font_handle.clone(),
        font_size: 24.0,
        color: Color::BLACK,
    };
    let text_alignment = TextAlignment::Center;

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    size: Size {
                        width: Val::Px(window.width()),
                        height: Val::Px(window.height()),
                    },
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
                        position: UiRect {
                            top: Val::Px(50.0),
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
                    format!("Music "),
                    text_style.clone(),
                )
                .with_text_alignment(text_alignment)
                .with_style(Style {
                    position: UiRect {
                        top: Val::Px(100.0),
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

            builder.spawn(NodeBundle {
                style: Style {
                    // size: Size::all(Val::Percent(100.)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    position: UiRect {
                        top: Val::Px(150.0),
                        ..default()
                    },
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }).with_children(|builder| {
                builder.spawn((
                    ButtonBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect {
                                right: Val::Px(24.0),
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
                        background_color: get_button_color(music_resource.playing),
                        ..default()
                    },
                    MainMenuItem::MusicOn,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "On",
                        text_style.clone(),
                    ));
                });

                builder.spawn((
                    ButtonBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect {
                                left: Val::Px(12.0),
                                right: Val::Px(12.0),
                                top: Val::Px(8.0),
                                bottom: Val::Px(8.0)
                            },
                            ..default()
                        },
                        background_color: get_button_color(!music_resource.playing),
                        ..default()
                    },
                    MainMenuItem::MusicOff,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Off",
                        text_style.clone(),
                    ));
                });
            });

            builder.spawn((
                TextBundle::from_section(
                    format!("Controls "),
                    text_style.clone(),
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

            builder.spawn(NodeBundle {
                style: Style {
                    // size: Size::all(Val::Percent(100.)),
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    position: UiRect {
                        top: Val::Px(230.0),
                        ..default()
                    },
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        ..default()
                    },
                    ..Default::default()
                },
                ..Default::default()
            }).with_children(|builder| {
                builder.spawn((
                    ButtonBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            margin: UiRect {
                                right: Val::Px(24.0),
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
                        background_color: get_button_color(control_scheme.is_mobile()),
                        ..default()
                    },
                    MainMenuItem::ControlMobile,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Mobile",
                        text_style.clone(),
                    ));
                });

                builder.spawn((
                    ButtonBundle {
                        style: Style {
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            padding: UiRect {
                                left: Val::Px(12.0),
                                right: Val::Px(12.0),
                                top: Val::Px(8.0),
                                bottom: Val::Px(8.0)
                            },
                            ..default()
                        },
                        background_color: get_button_color(control_scheme.is_desktop()),
                        ..default()
                    },
                    MainMenuItem::ControlDesktop,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Desktop",
                        text_style.clone(),
                    ));
                });
            });

            builder.spawn((
                TextBundle::from_section(
                    "Press M to toggle music",
                    text_style.clone(),
                )
                .with_text_alignment(text_alignment)
                .with_style(Style {
                    position: UiRect {
                        top: Val::Px(400.0),
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
