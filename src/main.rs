use animation::AnimationPlugin;
use bevy::{prelude::*, window::WindowResolution};

#[cfg(debug_assertions)]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use camera::CameraPlugin;
use debug::DebugPlugin;
use finish::FinishPlugin;
use gameover::GameOverPlugin;
use level_generator::LevelGeneratorPlugin;
use menu::MenuPlugin;
use music::MusicPlugin;
use obstacle::ObstaclePlugin;
use player::{PlayerPlugin};
use posts::PostsPlugin;
use sounds::{SoundPlugin, PostHitEvent};
use spatialtree::SpatialTree;
use stuneffect::StunPlugin;
use trail::TrailPlugin;
use tutorial::TutorialPlugin;
use uicontrols::UiControlsPlugin;
use yeti::YetiPlugin;

pub mod player;
pub mod obstacle;
pub mod collidable;
pub mod menu;
pub mod debug;
pub mod tutorial;
pub mod gameover;
pub mod trail;
pub mod posts;
pub mod uicontrols;
pub mod yeti;
pub mod animation;
pub mod stuneffect;
pub mod music;
pub mod sounds;
pub mod spatialtree;
pub mod camera;
pub mod level_generator;
pub mod finish;
/*
TODO
- sound
    - pridat zvuky pre yetiho a lyze
- collision detection
    - spravit spatial tree
- yeti
    1. pridat AI aby sa vyhybal prekazkam
    2. pridat animaciu ako zozerie hraca
    3. pridat walking animaciu na strany
 */

const SPATIAL_TILE_SIZE: f32 = 240.0;
const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;
pub const SPRITE_SIZE: f32 = 12.0;
pub const SCALE_FACTOR: f32 = 4.0;
const SELECTED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const NORMAL_BUTTON: Color = Color::rgb(1.0, 1.0, 1.0);

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    Playing,
    #[default]
    MainMenu,
    GameOver,
}

fn main() {
    let mut app = App::new();

    app
        .add_event::<PostHitEvent>()
        .insert_resource(ClearColor(Color::rgb(0.95, 0.95, 1.0)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(SCREEN_WIDTH, SCREEN_HEIGHT),
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }).set(ImagePlugin::default_nearest()))
        .insert_resource(SpatialTree::new(SPATIAL_TILE_SIZE))
        .add_state::<GameState>()
        .add_startup_system(setup)
        .add_plugin(MusicPlugin)
        .add_plugin(SoundPlugin)
        .add_plugin(TutorialPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(PlayerPlugin)
        .add_plugin(ObstaclePlugin)
        .add_plugin(PostsPlugin)
        .add_plugin(TrailPlugin)
        .add_plugin(YetiPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(StunPlugin)
        .add_plugin(GameOverPlugin)
        .add_plugin(CameraPlugin)
        .add_plugin(LevelGeneratorPlugin)
        .add_plugin(FinishPlugin);

    app.add_plugin(UiControlsPlugin);
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
    yeti_run: Vec<Rect>,
    yeti_fallen: Rect,
    yeti_step: Rect,
    stun: Rect,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let image_handle = asset_server.load("spritesheet.png");
    let font_handle = asset_server.load("QuinqueFive.ttf");

    let game_resource = GameResources {
        image_handle,
        font_handle,
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
        yeti_run: vec![
            Rect::new(10. * SPRITE_SIZE, 0., 11. * SPRITE_SIZE, SPRITE_SIZE),
            Rect::new(11. * SPRITE_SIZE, 0., 12. * SPRITE_SIZE, SPRITE_SIZE),
            Rect::new(12. * SPRITE_SIZE, 0., 13. * SPRITE_SIZE, SPRITE_SIZE),
        ],
        yeti_fallen: Rect::new(13. * SPRITE_SIZE, 0., 14. * SPRITE_SIZE, SPRITE_SIZE),
        yeti_step: Rect::new(14. * SPRITE_SIZE, 0., 15. * SPRITE_SIZE, SPRITE_SIZE),
        stun: Rect::new(15. * SPRITE_SIZE, 0., 16. * SPRITE_SIZE, SPRITE_SIZE),
    };

    commands.insert_resource(game_resource);
    commands.spawn(Camera2dBundle::default());
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
