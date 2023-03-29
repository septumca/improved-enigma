use bevy::prelude::*;


#[derive(Debug, Component)]
pub struct Collidable {
    width_half: f32,
    height_half: f32,
    offset_x: f32,
    offset_y: f32,
    left: f32,
    pub top: f32,
    right: f32,
    pub bottom: f32,
}

impl Collidable {
    pub fn new(x: f32, y: f32, width_half: f32, height_half: f32, offset_x: f32, offset_y: f32) -> Self {
        Self {
            width_half,
            height_half,
            offset_x,
            offset_y,
            left: x - width_half + offset_x,
            right: x + width_half + offset_x,
            top: y + height_half + offset_y,
            bottom: y - height_half + offset_y
        }
    }

    pub fn update_center(&mut self, x: f32, y: f32) {
        self.left = x - self.width_half + self.offset_x;
        self.right = x + self.width_half + self.offset_x;
        self.top = y + self.height_half + self.offset_y;
        self.bottom = y - self.height_half + self.offset_y;
    }

    pub fn intersect(&self, other: &Collidable) -> bool {
        !(self.right < other.left || other.right < self.left ||
        self.bottom > other.top || other.bottom > self.top)
    }
}
