use bevy::{prelude::*, window::WindowResolution};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use debug::DebugPlugin;
use gameover::GameOverPlugin;
use menu::MenuPlugin;
use obstacle::ObstaclePlugin;
use player::{PlayerPlugin};
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
- upravit zatacanie
    1. encapsulnut direction - aby enumy neleakovaly
    2. prerobit enumy na rotation f32
    3. extrahovat lyze do samostatnych obrazkov a child componentov
    4. na zaklade rotacie nastavit rotaciu lyzi a ich vzdialenost od seba
- upravit pohyb
    1. pre urcitu direction (po novom rotation) bude stanovena
         maximalna rychlost
         akceleracia
         deakceleracia
    2. pri zmene rotacie sa postupne (de)akceleruje na maximalnu rychlost
- upravit spawnovanie
    1. rozdelit priestor do vacsich gridov (e.g. 24x24)
    2. spawnovat obstacles v ramci gridu s random offsetom
    3. podobne aj s posts, vzdialenost bude v ramci policok
     (resp. sa vie vypocitat do ktorych policok spadaju posty a tam sa nevyspawnuje ziadna obstacle)
- pridat touch a mouse podporu
- yeti
 */

const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;
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
        .add_system(music_input)
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
    music_controller: Handle<AudioSink>,
    image_handle: Handle<Image>,
    font_handle: Handle<Font>,
    sprite_size: f32,
    down: Rect,
    sides: Vec<Rect>,
    fall_down: Rect,
    tree: Rect,
    stone: Rect,
    red_post: Rect,
    blue_post: Rect,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>
) {
    let image_handle = asset_server.load("spritesheet.png");
    let font_handle = asset_server.load("QuinqueFive.ttf");
    let music = asset_server.load("snow_globe_-_expanded.ogg");
    let music_controller = audio_sinks.get_handle(audio.play_with_settings(
        music,
        PlaybackSettings::LOOP.with_volume(0.25),
    ));
    let game_resource = GameResources {
        image_handle,
        font_handle,
        music_controller,
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
    };

    commands.insert_resource(game_resource);
    commands.spawn(Camera2dBundle::default());
}

fn music_input(
    keyboard_input: Res<Input<KeyCode>>,
    audio_sinks: Res<Assets<AudioSink>>,
    game_resource: Res<GameResources>,
) {
    if keyboard_input.just_pressed(KeyCode::M) {
        if let Some(sink) = audio_sinks.get(&game_resource.music_controller) {
            sink.toggle();
        }
    }
}

fn cleanup<T: Component>(
    mut commands: Commands,
    window: Query<&Window>,
    camera_q: Query<&Transform, With<Camera>>,
    t_q: Query<(Entity, &Transform), With<T>>
) {
    let Ok(transform_camera) = camera_q.get_single() else {
        return;
    };
    let Ok(window) = window.get_single() else {
        return;
    };
    let offset = window.height() * 0.6;

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
