use std::ops::{Add, Div, Mul, Sub};

use cgmath::{self, InnerSpace, VectorSpace, Zero};

use crate::Vec2;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CgmathVec2(pub cgmath::Vector2<f32>);

impl Add<CgmathVec2> for CgmathVec2 {
    type Output = CgmathVec2;

    fn add(self, other: CgmathVec2) -> CgmathVec2 {
        CgmathVec2(self.0 + other.0)
    }
}

impl Sub<CgmathVec2> for CgmathVec2 {
    type Output = CgmathVec2;

    fn sub(self, other: CgmathVec2) -> CgmathVec2 {
        CgmathVec2(self.0 - other.0)
    }
}

impl Mul<f32> for CgmathVec2 {
    type Output = CgmathVec2;

    fn mul(self, other: f32) -> CgmathVec2 {
        CgmathVec2(self.0 * other)
    }
}

impl Div<f32> for CgmathVec2 {
    type Output = CgmathVec2;

    fn div(self, other: f32) -> CgmathVec2 {
        CgmathVec2(self.0 / other)
    }
}

impl Vec2 for CgmathVec2 {
    fn x(&self) -> f32 {
        self.0.x
    }
    fn y(&self) -> f32 {
        self.0.y
    }

    fn set_x(&mut self, x: f32) {
        self.0.x = x;
    }
    fn set_y(&mut self, y: f32) {
        self.0.y = y;
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.0.x = x;
        self.0.y = y;
    }

    fn zero() -> Self {
        CgmathVec2(cgmath::Vector2::zero())
    }

    fn one() -> Self {
        CgmathVec2(cgmath::Vector2::new(cgmath::One::one(), cgmath::One::one()))
    }

    fn unit_x() -> Self {
        CgmathVec2(cgmath::Vector2::unit_x())
    }

    fn unit_y() -> Self {
        CgmathVec2(cgmath::Vector2::unit_y())
    }

    fn magnitude(&self) -> f32 {
        self.0.magnitude()
    }

    fn normalize(&mut self) {
        self.0 = self.0.normalize();
    }

    fn normalized(&self) -> Self {
        CgmathVec2(self.0.normalize())
    }

    fn dot(&self, other: &Self) -> f32 {
        self.0.dot(other.0)
    }

    fn cross(&self, other: &Self) -> f32 {
        self.0.x * other.0.y - self.0.y * other.0.x
    }

    fn angle(&self, other: &Self) -> f32 {
        self.dot(other).acos() / (self.magnitude() * other.magnitude())
    }

    fn distance(&self, other: &Self) -> f32 {
        (self.0 - other.0).magnitude()
    }

    fn lerp(&self, other: &Self, t: f32) -> Self {
        CgmathVec2(self.0.lerp(other.0, t))
    }

    fn reflect(&self, normal: &Self) -> Self {
        let d = self.dot(normal);
        CgmathVec2(self.0 - normal.0 * (2.0 * d))
    }

    fn rotated(&self, angle: f32) -> Self {
        let rot = cgmath::Matrix3::from_angle_z(cgmath::Rad(angle));
        CgmathVec2((rot * self.0.extend(0.0)).truncate())
    }

    fn rotate(&mut self, angle: f32) {
        let rot = cgmath::Matrix3::from_angle_z(cgmath::Rad(angle));
        self.0 = (rot * self.0.extend(0.0)).truncate();
    }

    fn to_array(&self) -> [f32; 2] {
        [self.0.x, self.0.y]
    }

    fn to_tuple(&self) -> (f32, f32) {
        (self.0.x, self.0.y)
    }

    fn from_slice(slice: &[f32]) -> Self {
        CgmathVec2(cgmath::Vector2::new(slice[0], slice[1]))
    }

    fn from_tuple(t: (f32, f32)) -> Self {
        CgmathVec2(cgmath::Vector2::new(t.0, t.1))
    }
}
