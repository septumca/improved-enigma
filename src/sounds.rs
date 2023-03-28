use bevy::{prelude::*};

use crate::GameState;

#[derive(Resource)]
struct SoundsResource {
    post_hit: Handle<AudioSource>,
    sound_on: bool,
}

pub struct PostHitEvent;

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup)
            .add_systems((
                on_post_hit,
            ).in_set(OnUpdate(GameState::Playing)))
            .add_system(sound_control);
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.insert_resource(SoundsResource {
        post_hit: asset_server.load("Rise01.ogg"),
        sound_on: true,
    });
}

fn on_post_hit(
    audio: Res<Audio>,
    sound_resource: Res<SoundsResource>,
    mut ev_posthit: EventReader<PostHitEvent>,
) {
    for _ in ev_posthit.iter() {
        if sound_resource.sound_on {
            audio.play(sound_resource.post_hit.clone());
        }
    }
}

fn sound_control(
    mut sound_resource: ResMut<SoundsResource>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if keyboard_input.just_pressed(KeyCode::M) {
        sound_resource.sound_on = !sound_resource.sound_on;
    }
}