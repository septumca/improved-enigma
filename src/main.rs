use bevy::{prelude::*, window::WindowResolution};

use obstacle::ObstaclePlugin;
use player::PlayerPlugin;

pub mod player;
pub mod obstacle;
pub mod collidable;
/*
TODO
- zprehladnit rozdelenie kodu do pluginov
- pridat reset button a rozne states
https://bevy-cheatbook.github.io/programming/states.html
https://github.com/mwbryant/logic-turn-based-rpg/blob/devlog2/src/combat/turn_based.rs
https://github.com/bevyengine/bevy/blob/main/examples/games/breakout.rs
 */

pub const SCREEN_WIDTH: f32 = 640.0;
pub const SCREEN_HEIGHT: f32 = 640.0;
pub const SPRITE_SIZE: f32 = 12.0;
pub const SCALE_FACTOR: f64 = 6.0;


#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Playing,
}

fn main() {
    App::new()

        .insert_resource(GameResources::new())
        .insert_resource(ClearColor(Color::rgb(0.95, 0.95, 1.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(SCREEN_WIDTH, SCREEN_HEIGHT).with_scale_factor_override(SCALE_FACTOR),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_state::<GameState>()
        .add_startup_system(setup)
        .add_plugin(PlayerPlugin)
        .add_plugin(ObstaclePlugin)
        .run();
}

#[derive(Component)]
pub struct Alive;

#[derive(Resource)]
struct GameResources {
    image_handle: Option<Handle<Image>>,
    font_handle: Option<Handle<Font>>,
    sprite_size: f32,
    down: Rect,
    sides: Vec<Rect>,
    fall_down: Rect,
    tree: Rect,
    stone: Rect,
}

impl GameResources {
    pub fn new() -> Self {
        let sprite_size = SPRITE_SIZE;
        Self {
            image_handle: None,
            font_handle: None,
            sprite_size,
            down: Rect::new(0. * sprite_size, 0., 1. * sprite_size, sprite_size),
            sides: vec![
                Rect::new(1. * sprite_size, 0., 2. * sprite_size, sprite_size),
                Rect::new(2. * sprite_size, 0., 3. * sprite_size, sprite_size),
                Rect::new(3. * sprite_size, 0., 4. * sprite_size, sprite_size),
                Rect::new(4. * sprite_size, 0., 5. * sprite_size, sprite_size),
            ],
            fall_down: Rect::new(5. * sprite_size, 0., 6. * sprite_size, sprite_size),
            tree: Rect::new(6. * sprite_size, 0., 7. * sprite_size, sprite_size),
            stone: Rect::new(7. * sprite_size, 0., 8. * sprite_size, sprite_size),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut game_resources: ResMut<GameResources>,
    asset_server: Res<AssetServer>
) {
    let texture_handle = asset_server.load("spritesheet.png");
    let font = asset_server.load("QuinqueFive.ttf");
    game_resources.image_handle = Some(texture_handle.clone());
    game_resources.font_handle = Some(font.clone());

    commands.spawn(Camera2dBundle::default());
}


fn cleanup<T: Component>(
    mut commands: Commands,
    camera_q: Query<&Transform, With<Camera>>,
    obstacles_q: Query<(Entity, &Transform), With<T>>
) {
    let Ok(transform_camera) = camera_q.get_single() else {
        return;
    };
    let offset = SCREEN_HEIGHT / SCALE_FACTOR as f32 * 0.6;

    for (entity, transform_obstacle) in obstacles_q.iter() {
        if transform_obstacle.translation.y - transform_camera.translation.y > offset {
            commands.entity(entity).despawn_recursive();
        }
    }
}
