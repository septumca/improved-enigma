use bevy::prelude::*;


#[derive(Debug, Component)]
pub struct Collidable {
    width_half: f32,
    height_half: f32,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
}

impl Collidable {
    pub fn new(x: f32, y: f32, width_half: f32, height_half: f32) -> Self {
        Self {
            width_half,
            height_half,
            left: x - width_half,
            right: x + width_half,
            top: y + height_half,
            bottom: y - height_half
        }
    }

    pub fn update_center(&mut self, x: f32, y: f32) {
        self.left = x - self.width_half;
        self.right = x + self.width_half;
        self.top = y + self.height_half;
        self.bottom = y - self.height_half;
    }

    pub fn intersect(&self, other: &Collidable) -> bool {
        !(self.right < other.left || other.right < self.left ||
        self.bottom > other.top || other.bottom > self.top)
    }
}
