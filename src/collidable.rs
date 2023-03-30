use bevy::{prelude::*, math::vec2};


//  // a1 is line1 start, a2 is line1 end, b1 is line2 start, b2 is line2 end
// static bool Intersects(Vector2 a1, Vector2 a2, Vector2 b1, Vector2 b2, out Vector2 intersection)
// {
//     intersection = Vector2.Zero;

//     Vector2 b = a2 - a1;
//     Vector2 d = b2 - b1;
//     float bDotDPerp = b.X * d.Y - b.Y * d.X;

//     // if b dot d == 0, it means the lines are parallel so have infinite intersection points
//     if (bDotDPerp == 0)
//         return false;

//     Vector2 c = b1 - a1;
//     float t = (c.X * d.Y - c.Y * d.X) / bDotDPerp;
//     if (t < 0 || t > 1)
//         return false;

//     float u = (c.X * b.Y - c.Y * b.X) / bDotDPerp;
//     if (u < 0 || u > 1)
//         return false;

//     intersection = a1 + t * b;

//     return true;
// }

fn line_intersection(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2) -> Option<Vec2> {
    let b = a2 - a1;
    let d = b2 - b1;
    let b_dot_d = b.perp_dot(d);

    if b_dot_d == 0.0 {
        return None;
    }
    let c = b1 - a1;
    let t = c.dot(d) / b_dot_d;
    if t < 0.0 || t > 1.0 {
        return None;
    }
    let u = c.dot(b) / b_dot_d;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    Some(a1 + t * b)
}

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

#[derive(Debug, Component)]
pub struct CollidableMovable;

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

    pub fn intersect_line(&self, line_start: Vec2, line_end: Vec2) -> Option<Vec2> {
        let edge1 = vec2(self.top, self.left);
        let edge2 = vec2(self.top, self.right);
        if let Some(intersection) = line_intersection(edge1, edge2, line_start, line_end) {
            return Some(intersection);
        }
        let edge1 = vec2(self.top, self.left);
        let edge2 = vec2(self.bottom, self.left);
        if let Some(intersection) = line_intersection(edge1, edge2, line_start, line_end) {
            return Some(intersection);
        }
        let edge1 = vec2(self.bottom, self.left);
        let edge2 = vec2(self.bottom, self.right);
        if let Some(intersection) = line_intersection(edge1, edge2, line_start, line_end) {
            return Some(intersection);
        }
        let edge1 = vec2(self.top, self.right);
        let edge2 = vec2(self.bottom, self.right);
        if let Some(intersection) = line_intersection(edge1, edge2, line_start, line_end) {
            return Some(intersection);
        }

        None
    }
}
