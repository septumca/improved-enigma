use bevy::{prelude::*};

use crate::{GameState};


const ANIMATION_TIMER: f32 = 8.0 / 60.0;

#[derive(Component)]
pub struct Animation {
    pub timer: Timer,
    frames: Vec<Rect>,
    pub act_frame_index: usize,
}

#[derive(Component)]
pub struct AnimateRotation {
    pub angular_vel: f32,
}


impl Animation {
    pub fn new(frames: Vec<Rect>, mode: TimerMode) -> Self {
        Self {
            act_frame_index: 0,
            timer: Timer::from_seconds(ANIMATION_TIMER, mode),
            frames
        }
    }
    pub fn next_frame(&mut self) {
        if self.act_frame_index == self.frames.len() - 1 && self.timer.mode() == TimerMode::Repeating {
            self.act_frame_index = 0;
            return;
        }
        self.act_frame_index += 1;
    }

    pub fn get_frame(&self) -> Option<Rect> {
        self.frames.get(self.act_frame_index).and_then(|f| Some(f.clone()))
    }

    pub fn set_frames(&mut self, frames: Vec<Rect>) {
        self.frames = frames;
        self.timer.reset();
        self.act_frame_index = 0;
    }
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(
                (
                    update_animation,
                    update_animate_rotation,
                ).in_set(OnUpdate(GameState::Playing)));
    }
}

fn update_animation(
    timer: Res<Time>,
    mut anim_q: Query<(&mut Sprite, &mut Animation)>
) {
    let dt = timer.delta();
    for (mut sprite, mut animation) in anim_q.iter_mut() {
        if animation.timer.tick(dt).just_finished() {
            animation.next_frame();
            if let Some(rect) = animation.get_frame() {
                sprite.rect = Some(rect);
            }
        }
    }
}

fn update_animate_rotation(
    timer: Res<Time>,
    mut anim_q: Query<(&mut Transform, &AnimateRotation)>
) {
    for (mut transform, animate_rotation) in anim_q.iter_mut() {
        transform.rotate(Quat::from_rotation_z(animate_rotation.angular_vel * timer.delta_seconds()));
    }
}
