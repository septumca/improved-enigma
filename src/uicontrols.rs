use bevy::{prelude::*};
use os_info::Type;

use crate::{GameState, despawn, GameResources, player::{Player}, Alive, SELECTED_BUTTON};

#[derive(Component)]
pub struct UiControls;

#[derive(Component, Clone)]
pub enum UiControlType {
    Left,
    Right,
}

#[derive(PartialEq)]
pub enum ControlSchemeType {
    Mobile,
    Desktop,
}

#[derive(Resource)]
pub struct  ControlScheme {
    pub kind: ControlSchemeType,
}

impl ControlScheme {
    pub fn set_mobile(&mut self) {
        self.kind = ControlSchemeType::Mobile;
    }

    pub fn set_desktop(&mut self) {
        self.kind = ControlSchemeType::Desktop;
    }


    pub fn is_mobile(&self) -> bool {
        self.kind == ControlSchemeType::Mobile
    }

    pub fn is_desktop(&self) -> bool {
        self.kind == ControlSchemeType::Desktop
    }
}


pub struct UiControlsPlugin;

impl Plugin for UiControlsPlugin {
    fn build(&self, app: &mut App) {
        let control_scheme_kind = if os_info::get().os_type() == Type::Unknown {
            ControlSchemeType::Mobile
        } else {
            ControlSchemeType::Desktop
        };
        app
            .insert_resource(ControlScheme {
                kind: control_scheme_kind
            })
            .add_system(setup_uicontrols.in_schedule(OnEnter(GameState::Playing)))
            .add_system(despawn::<UiControls>.in_schedule(OnExit(GameState::Playing)))
            .add_system(player_input.in_set(OnUpdate(GameState::Playing)));
    }
}


pub fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut player_q: Query<&mut Player, With<Alive>>,
    interaction_query: Query<
        (&Interaction, &UiControlType),
        (With<Button>, With<UiControls>),
    >,
) {
    let Ok(mut player) = player_q.get_single_mut() else {
        return;
    };

    if keyboard_input.pressed(KeyCode::A) {
        player.control_type = Some(UiControlType::Left);
    }
    if keyboard_input.pressed(KeyCode::D) {
        player.control_type = Some(UiControlType::Right);
    }

    for (interaction, uicontrol_type) in &interaction_query {
        match *interaction {
            Interaction::Clicked | Interaction::Hovered => {
                player.control_type = Some(uicontrol_type.clone());
            },
            _ => {}
        }
    }
}

fn setup_uicontrols(
    mut commands: Commands,
    window: Query<&Window>,
    game_resources: Res<GameResources>,
    control_scheme: Res<ControlScheme>,
) {
    if control_scheme.kind == ControlSchemeType::Desktop {
        return;
    }
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
            background_color: SELECTED_BUTTON.into(),
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
            background_color: SELECTED_BUTTON.into(),
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