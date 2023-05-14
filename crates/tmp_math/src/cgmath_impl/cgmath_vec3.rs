use std::ops::{Add, Div, Mul, Sub};

use crate::Vec3;
use cgmath::{self, InnerSpace, Rotation3, VectorSpace};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct CgmathVec3(pub cgmath::Vector3<f32>);

impl Add<CgmathVec3> for CgmathVec3 {
    type Output = CgmathVec3;

    fn add(self, other: CgmathVec3) -> CgmathVec3 {
        CgmathVec3(self.0 + other.0)
    }
}

impl Sub<CgmathVec3> for CgmathVec3 {
    type Output = CgmathVec3;

    fn sub(self, other: CgmathVec3) -> CgmathVec3 {
        CgmathVec3(self.0 - other.0)
    }
}

impl Mul<f32> for CgmathVec3 {
    type Output = CgmathVec3;

    fn mul(self, other: f32) -> CgmathVec3 {
        CgmathVec3(self.0 * other)
    }
}

impl Div<f32> for CgmathVec3 {
    type Output = CgmathVec3;

    fn div(self, other: f32) -> CgmathVec3 {
        CgmathVec3(self.0 / other)
    }
}

impl Vec3 for CgmathVec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        CgmathVec3(cgmath::Vector3::new(x, y, z))
    }

    fn new_from_slice(slice: &[f32]) -> Self {
        CgmathVec3(cgmath::Vector3::new(slice[0], slice[1], slice[2]))
    }

    fn new_from_tuple(t: (f32, f32, f32)) -> Self {
        CgmathVec3(cgmath::Vector3::new(t.0, t.1, t.2))
    }

    fn x(&self) -> f32 {
        self.0.x
    }
    fn y(&self) -> f32 {
        self.0.y
    }
    fn z(&self) -> f32 {
        self.0.z
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

    fn set_xy(&mut self, x: f32, y: f32) {
        self.0.x = x;
        self.0.y = y;
    }
    fn set_xz(&mut self, x: f32, z: f32) {
        self.0.x = x;
        self.0.z = z;
    }
    fn set_yz(&mut self, y: f32, z: f32) {
        self.0.y = y;
        self.0.z = z;
    }

    fn set_xyz(&mut self, x: f32, y: f32, z: f32) {
        self.0.x = x;
        self.0.y = y;
        self.0.z = z;
    }

    fn zero() -> Self {
        CgmathVec3(cgmath::Zero::zero())
    }
    fn one() -> Self {
        CgmathVec3(cgmath::Vector3::new(
            cgmath::One::one(),
            cgmath::One::one(),
            cgmath::One::one(),
        ))
    }
    fn unit_x() -> Self {
        CgmathVec3(cgmath::Vector3::unit_x())
    }
    fn unit_y() -> Self {
        CgmathVec3(cgmath::Vector3::unit_y())
    }
    fn unit_z() -> Self {
        CgmathVec3(cgmath::Vector3::unit_z())
    }
    fn magnitude(&self) -> f32 {
        self.0.magnitude()
    }
    fn normalize(&mut self) {
        self.0 = self.0.normalize();
    }
    fn normalized(&self) -> Self {
        CgmathVec3(self.0.normalize())
    }
    fn dot(&self, other: &Self) -> f32 {
        cgmath::dot(self.0, other.0)
    }
    fn cross(&self, other: &Self) -> Self {
        CgmathVec3(self.0.cross(other.0))
    }

    fn reflect(&self, normal: &Self) -> Self {
        let d = 2.0 * self.dot(normal);
        CgmathVec3(self.0 - (normal.0 * d))
    }

    fn lerp(&self, other: &Self, amount: f32) -> Self {
        CgmathVec3(self.0.lerp(other.0, amount))
    }

    fn distance(&self, other: &Self) -> f32 {
        (self.0 - other.0).magnitude()
    }

    fn angle(&self, other: &Self) -> f32 {
        self.dot(other).acos() / (self.magnitude() * other.magnitude())
    }

    fn project(&self, onto: &Self) -> Self {
        let onto = onto.normalized();
        let d = self.dot(&onto);
        onto * d
    }

    fn rotate_x(&mut self, angle: f32) {
        let rot = cgmath::Matrix3::from_angle_x(cgmath::Rad(angle));
        self.0 = rot * self.0;
    }

    fn rotated_x(&self, angle: f32) -> Self {
        let rot = cgmath::Matrix3::from_angle_x(cgmath::Rad(angle));
        CgmathVec3(rot * self.0)
    }

    fn rotate_y(&mut self, angle: f32) {
        let rot = cgmath::Matrix3::from_angle_y(cgmath::Rad(angle));
        self.0 = rot * self.0;
    }

    fn rotated_y(&self, angle: f32) -> Self {
        let rot = cgmath::Matrix3::from_angle_y(cgmath::Rad(angle));
        CgmathVec3(rot * self.0)
    }

    fn rotate_z(&mut self, angle: f32) {
        let rot = cgmath::Matrix3::from_angle_z(cgmath::Rad(angle));
        self.0 = rot * self.0;
    }

    fn rotated_z(&self, angle: f32) -> Self {
        let rot = cgmath::Matrix3::from_angle_z(cgmath::Rad(angle));
        CgmathVec3(rot * self.0)
    }

    fn rotate(&mut self, axis: &Self, angle: f32) {
        let rot = cgmath::Quaternion::from_axis_angle(axis.0, cgmath::Rad(angle));
        self.0 = rot * self.0;
    }

    fn rotated(&self, axis: &Self, angle: f32) -> Self {
        let rot = cgmath::Quaternion::from_axis_angle(axis.0, cgmath::Rad(angle));
        CgmathVec3(rot * self.0)
    }

    fn to_array(&self) -> [f32; 3] {
        [self.0.x, self.0.y, self.0.z]
    }

    fn to_tuple(&self) -> (f32, f32, f32) {
        (self.0.x, self.0.y, self.0.z)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cgmath_vector3() {
        let mut v = CgmathVec3::zero();
        assert_eq!(v.x(), 0.0);
        assert_eq!(v.y(), 0.0);
        assert_eq!(v.z(), 0.0);

        v.set_x(1.0);
        v.set_y(2.0);
        v.set_z(3.0);
        assert_eq!(v.x(), 1.0);
        assert_eq!(v.y(), 2.0);
        assert_eq!(v.z(), 3.0);

        v.set_xy(4.0, 5.0);
        assert_eq!(v.x(), 4.0);
        assert_eq!(v.y(), 5.0);
        assert_eq!(v.z(), 3.0);

        v.set_xz(6.0, 7.0);
        assert_eq!(v.x(), 6.0);
        assert_eq!(v.y(), 5.0);
        assert_eq!(v.z(), 7.0);

        v.set_yz(8.0, 9.0);
        assert_eq!(v.x(), 6.0);
        assert_eq!(v.y(), 8.0);
        assert_eq!(v.z(), 9.0);

        v.set_xyz(10.0, 11.0, 12.0);
        assert_eq!(v.x(), 10.0);
        assert_eq!(v.y(), 11.0);
        assert_eq!(v.z(), 12.0);

        assert_eq!(v.magnitude(), 19.131126);
        v.normalize();
        assert_eq!(v.magnitude(), 1.0);
        assert_eq!(v.x(), 0.5240975);
        assert_eq!(v.y(), 0.57604843);
        assert_eq!(v.z(), 0.62799937);

        let v2 = v.normalized();
        assert_eq!(v2.magnitude(), 1.0);
        assert_eq!(v2.x(), 0.5240975);
        assert_eq!(v2.y(), 0.57604843);
        assert_eq!(v2.z(), 0.62799937);

        let v3 = CgmathVec3::one();
        assert_eq!(v3.x(), 1 as f32);
        assert_eq!(v3.y(), 1 as f32);
        assert_eq!(v3.z(), 1 as f32);

        let v4 = CgmathVec3::unit_x();
        assert_eq!(v4.x(), 1 as f32);
        assert_eq!(v4.y(), 0 as f32);
        assert_eq!(v4.z(), 0 as f32);
    }
}
