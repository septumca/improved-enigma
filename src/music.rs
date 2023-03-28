use bevy::{prelude::*};

#[derive(Resource)]
struct MusicResource {
    music_controller: Handle<AudioSink>,
}

pub struct MusicPlugin;

impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_system(music_control);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    audio_sinks: Res<Assets<AudioSink>>
) {
    let audio_handle = asset_server.load("music.ogg");
    let music_controller = audio_sinks.get_handle(audio.play_with_settings(
        audio_handle,
        PlaybackSettings::LOOP.with_volume(0.25),
    ));

    commands.insert_resource(MusicResource {
        music_controller,
    });
}

fn music_control(
    music_resource: Res<MusicResource>,
    keyboard_input: Res<Input<KeyCode>>,
    audio_sinks: Res<Assets<AudioSink>>,
) {
    if keyboard_input.just_pressed(KeyCode::M) {
        if let Some(sink) = audio_sinks.get(&music_resource.music_controller) {
            sink.toggle();
        }
    }
}