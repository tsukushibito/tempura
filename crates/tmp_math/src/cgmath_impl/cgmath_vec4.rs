use crate::Vec4;
use cgmath::{self, InnerSpace, VectorSpace, Zero};
use std::ops::{Add, Div, Mul, Sub};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CgmathVec4(pub cgmath::Vector4<f32>);

impl Add<CgmathVec4> for CgmathVec4 {
    type Output = CgmathVec4;

    fn add(self, other: CgmathVec4) -> CgmathVec4 {
        CgmathVec4(self.0 + other.0)
    }
}

impl Sub<CgmathVec4> for CgmathVec4 {
    type Output = CgmathVec4;

    fn sub(self, other: CgmathVec4) -> CgmathVec4 {
        CgmathVec4(self.0 - other.0)
    }
}

impl Mul<f32> for CgmathVec4 {
    type Output = CgmathVec4;

    fn mul(self, other: f32) -> CgmathVec4 {
        CgmathVec4(self.0 * other)
    }
}

impl Div<f32> for CgmathVec4 {
    type Output = CgmathVec4;

    fn div(self, other: f32) -> CgmathVec4 {
        CgmathVec4(self.0 / other)
    }
}

impl Vec4 for CgmathVec4 {
    fn x(&self) -> f32 {
        self.0.x
    }
    fn y(&self) -> f32 {
        self.0.y
    }
    fn z(&self) -> f32 {
        self.0.z
    }
    fn w(&self) -> f32 {
        self.0.w
    }

    fn set_x(&mut self, x: f32) {
        self.0.x = x;
    }
    fn set_y(&mut self, y: f32) {
        self.0.y = y;
    }
    fn set_z(&mut self, z: f32) {
        self.0.z = z;
    }
    fn set_w(&mut self, w: f32) {
        self.0.w = w;
    }

    fn set_xy(&mut self, x: f32, y: f32) {
        self.0.x = x;
        self.0.y = y;
    }
    fn set_xz(&mut self, x: f32, z: f32) {
        self.0.x = x;
        self.0.z = z;
    }
    fn set_xw(&mut self, x: f32, w: f32) {
        self.0.x = x;
        self.0.w = w;
    }
    fn set_yz(&mut self, y: f32, z: f32) {
        self.0.y = y;
        self.0.z = z;
    }
    fn set_yw(&mut self, y: f32, w: f32) {
        self.0.y = y;
        self.0.w = w;
    }
    fn set_zw(&mut self, z: f32, w: f32) {
        self.0.z = z;
        self.0.w = w;
    }

    fn set_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.0.x = x;
        self.0.y = y;
        self.0.z = z;
    }
    fn set_xyw(&mut self, x: f32, y: f32, w: f32) {
        self.0.x = x;
        self.0.y = y;
        self.0.w = w;
    }
    fn set_xzw(&mut self, x: f32, z: f32, w: f32) {
        self.0.x = x;
        self.0.z = z;
        self.0.w = w;
    }
    fn set_yzw(&mut self, y: f32, z: f32, w: f32) {
        self.0.y = y;
        self.0.z = z;
        self.0.w = w;
    }

    fn set_xyzw(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.0.x = x;
        self.0.y = y;
        self.0.z = z;
        self.0.w = w;
    }

    fn zero() -> Self {
        CgmathVec4(cgmath::Vector4::zero())
    }

    fn one() -> Self {
        CgmathVec4(cgmath::Vector4::new(
            cgmath::One::one(),
            cgmath::One::one(),
            cgmath::One::one(),
            cgmath::One::one(),
        ))
    }

    fn unit_x() -> Self {
        CgmathVec4(cgmath::Vector4::unit_x())
    }

    fn unit_y() -> Self {
        CgmathVec4(cgmath::Vector4::unit_y())
    }

    fn unit_z() -> Self {
        CgmathVec4(cgmath::Vector4::unit_z())
    }

    fn unit_w() -> Self {
        CgmathVec4(cgmath::Vector4::unit_w())
    }

    fn magnitude(&self) -> f32 {
        self.0.magnitude()
    }

    fn normalize(&mut self) {
        self.0 = self.0.normalize();
    }

    fn normalized(&self) -> Self {
        CgmathVec4(self.0.normalize())
    }

    fn dot(&self, other: &Self) -> f32 {
        self.0.dot(other.0)
    }

    fn distance(&self, other: &Self) -> f32 {
        (self.0 - other.0).magnitude()
    }

    fn lerp(&self, other: &Self, t: f32) -> Self {
        CgmathVec4(self.0.lerp(other.0, t))
    }
}
