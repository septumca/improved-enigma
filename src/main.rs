use bevy::{prelude::*, window::WindowResolution};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use debug::DebugPlugin;
use gameover::GameOverPlugin;
use menu::MenuPlugin;
use obstacle::ObstaclePlugin;
use player::{PlayerPlugin, Player};
use posts::PostsPlugin;
use trail::TrailPlugin;
use tutorial::TutorialPlugin;

pub mod player;
pub mod obstacle;
pub mod collidable;
pub mod menu;
pub mod debug;
pub mod tutorial;
pub mod gameover;
pub mod trail;
pub mod posts;
/*
TODO
- cely obstacle spawner nech je ako iterator a nech sa initialuzuje v setup funkcii po player::setup
- upravit Z indexy
- pridat touch a mouse podporu
- Zimplementovat a nakreslit yetiho
 */

pub const SCREEN_WIDTH: f32 = 640.0;
pub const SCREEN_HEIGHT: f32 = 640.0;
pub const SPRITE_SIZE: f32 = 12.0;
pub const SCALE_FACTOR: f32 = 4.0;


#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    MainMenu,
    Playing,
    GameOver,
}

fn main() {
    let mut app = App::new();
    app
        .insert_resource(GameResources::new())
        .insert_resource(ClearColor(Color::rgb(0.95, 0.95, 1.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(SCREEN_WIDTH, SCREEN_HEIGHT),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .add_state::<GameState>()
        .add_startup_system(setup)
        .add_plugin(TutorialPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(ObstaclePlugin)
        .add_plugin(PostsPlugin)
        .add_plugin(TrailPlugin)
        .add_plugin(GameOverPlugin);


    #[cfg(debug_assertions)]
    {
        app.add_plugin(WorldInspectorPlugin::new());
        app.add_plugin(DebugPlugin);
    }

    app.run();
}

#[derive(Component)]
pub struct Alive;


#[derive(Resource)]
pub struct GameResources {
    image_handle: Option<Handle<Image>>,
    font_handle: Option<Handle<Font>>,
    sprite_size: f32,
    down: Rect,
    sides: Vec<Rect>,
    fall_down: Rect,
    tree: Rect,
    stone: Rect,
    red_post: Rect,
    blue_post: Rect,
}

impl GameResources {
    pub fn new() -> Self {
        Self {
            image_handle: None,
            font_handle: None,
            sprite_size: SPRITE_SIZE * SCALE_FACTOR,
            down: Rect::new(0. * SPRITE_SIZE, 0., 1. * SPRITE_SIZE, SPRITE_SIZE),
            sides: vec![
                Rect::new(1. * SPRITE_SIZE, 0., 2. * SPRITE_SIZE, SPRITE_SIZE),
                Rect::new(2. * SPRITE_SIZE, 0., 3. * SPRITE_SIZE, SPRITE_SIZE),
                Rect::new(3. * SPRITE_SIZE, 0., 4. * SPRITE_SIZE, SPRITE_SIZE),
                Rect::new(4. * SPRITE_SIZE, 0., 5. * SPRITE_SIZE, SPRITE_SIZE),
            ],
            fall_down: Rect::new(5. * SPRITE_SIZE, 0., 6. * SPRITE_SIZE, SPRITE_SIZE),
            tree: Rect::new(6. * SPRITE_SIZE, 0., 7. * SPRITE_SIZE, SPRITE_SIZE),
            stone: Rect::new(7. * SPRITE_SIZE, 0., 8. * SPRITE_SIZE, SPRITE_SIZE),
            red_post: Rect::new(8. * SPRITE_SIZE, 0., 9. * SPRITE_SIZE, SPRITE_SIZE),
            blue_post: Rect::new(9. * SPRITE_SIZE, 0., 10. * SPRITE_SIZE, SPRITE_SIZE),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut game_resources: ResMut<GameResources>,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>
) {
    let texture_handle = asset_server.load("spritesheet.png");
    let font = asset_server.load("QuinqueFive.ttf");
    let music = asset_server.load("snow_globe_-_expanded.ogg");
    game_resources.image_handle = Some(texture_handle.clone());
    game_resources.font_handle = Some(font.clone());

    audio.play_with_settings(
        music,
        PlaybackSettings::LOOP.with_volume(0.25),
    );
    commands.spawn(Camera2dBundle::default());
}


fn cleanup<T: Component>(
    mut commands: Commands,
    camera_q: Query<&Transform, With<Camera>>,
    t_q: Query<(Entity, &Transform), With<T>>
) {
    let Ok(transform_camera) = camera_q.get_single() else {
        return;
    };
    let offset = SCREEN_HEIGHT * 0.6;

    for (entity, transform_t) in t_q.iter() {
        if transform_camera.translation.y + offset < transform_t.translation.y {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn despawn<T: Component>(
    mut commands: Commands,
    components_q: Query<Entity, With<T>>
) {
    for entity in components_q.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
