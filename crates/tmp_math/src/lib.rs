mod cgmath_impl;

#[cfg(feature = "cgmath")]
pub use cgmath_impl::*;

pub trait Vec2:
    Copy
    + Clone
    + std::fmt::Debug
    + PartialEq
    + Sized
    + std::ops::Add<Self, Output = Self>
    + std::ops::Sub<Self, Output = Self>
    + std::ops::Mul<f32, Output = Self>
    + std::ops::Div<f32, Output = Self>
{
    fn x(&self) -> f32;
    fn y(&self) -> f32;

    fn set_x(&mut self, x: f32);
    fn set_y(&mut self, y: f32);

    fn set_xy(&mut self, x: f32, y: f32);

    fn zero() -> Self;
    fn one() -> Self;
    fn unit_x() -> Self;
    fn unit_y() -> Self;
    fn magnitude(&self) -> f32;
    fn normalize(&mut self);
    fn normalized(&self) -> Self;
    fn dot(&self, other: &Self) -> f32;
    fn cross(&self, other: &Self) -> f32;
    fn angle(&self, other: &Self) -> f32;
    fn distance(&self, other: &Self) -> f32;
    fn lerp(&self, other: &Self, t: f32) -> Self;
    fn reflect(&self, normal: &Self) -> Self;
    fn rotated(&self, angle: f32) -> Self;
    fn rotate(&mut self, angle: f32);
    fn to_array(&self) -> [f32; 2];
    fn to_tuple(&self) -> (f32, f32);
    fn from_slice(slice: &[f32]) -> Self;
    fn from_tuple(t: (f32, f32)) -> Self;
}

/// A 3D vector
pub trait Vec3:
    Copy
    + Clone
    + std::fmt::Debug
    + PartialEq
    + Sized
    + std::ops::Add<Self, Output = Self>
    + std::ops::Sub<Self, Output = Self>
    + std::ops::Mul<f32, Output = Self>
    + std::ops::Div<f32, Output = Self>
{
    fn new(x: f32, y: f32, z: f32) -> Self;
    fn new_from_slice(slice: &[f32]) -> Self;
    fn new_from_tuple(t: (f32, f32, f32)) -> Self;

    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn z(&self) -> f32;

    fn set_x(&mut self, x: f32);
    fn set_y(&mut self, y: f32);
    fn set_z(&mut self, z: f32);

    fn set_xy(&mut self, x: f32, y: f32);
    fn set_xz(&mut self, x: f32, z: f32);
    fn set_yz(&mut self, y: f32, z: f32);

    fn set_xyz(&mut self, x: f32, y: f32, z: f32);

    // This creates a new Vector3 object with all values set to 0.
    fn zero() -> Self;
    // This creates a new Vector3 object with all values set to 1.
    fn one() -> Self;
    // This creates a new Vector3 object with the x value set to 1 and the y and z values set to 0.
    fn unit_x() -> Self;
    // This creates a new Vector3 object with the y value set to 1 and the x and z values set to 0.
    fn unit_y() -> Self;
    // This creates a new Vector3 object with the z value set to 1 and the x and y values set to 0.
    fn unit_z() -> Self;
    // This returns the magnitude (or length) of the Vector3 object.
    fn magnitude(&self) -> f32;
    // This normalizes the Vector3 object, which means that its magnitude becomes 1.
    fn normalize(&mut self);
    // This returns the Vector3 object normalized, which means that its magnitude becomes 1.
    fn normalized(&self) -> Self;
    // This returns the dot product of two Vector3 objects.
    fn dot(&self, other: &Self) -> f32;
    // This returns the cross product of two Vector3 objects.
    fn cross(&self, other: &Self) -> Self;
    // This returns the reflection of the Vector3 object around a given normal vector.
    fn reflect(&self, normal: &Self) -> Self;
    // This performs a linear interpolation between two Vector3 objects.
    fn lerp(&self, other: &Self, t: f32) -> Self;
    // This returns the distance between two Vector3 objects.
    fn distance(&self, other: &Self) -> f32;
    // This returns the angle between two Vector3 objects.
    fn angle(&self, other: &Self) -> f32;
    // This returns the projection of the Vector3 object onto another vector.
    fn project(&self, onto: &Self) -> Self;
    // This rotates the Vector3 object around the x-axis by a given angle.
    fn rotate_x(&mut self, angle: f32);
    // This return Vector3 object rotated around the x-axis by a given angle.
    fn rotated_x(&self, angle: f32) -> Self;
    // This rotates the Vector3 object around the y-axis by a given angle.
    fn rotate_y(&mut self, angle: f32);
    // This rotates the Vector3 object around the y-axis by a given angle.
    fn rotated_y(&self, angle: f32) -> Self;
    // This rotates the Vector3 object around the z-axis by a given angle.
    fn rotate_z(&mut self, angle: f32);
    // This rotates the Vector3 object around the z-axis by a given angle.
    fn rotated_z(&self, angle: f32) -> Self;
    // This rotates the Vector3 object around a given axis by a given angle.
    fn rotate(&mut self, axis: &Self, angle: f32);
    // This rotates the Vector3 object around a given axis by a given angle.
    fn rotated(&self, axis: &Self, angle: f32) -> Self;

    fn to_array(&self) -> [f32; 3];
    fn to_tuple(&self) -> (f32, f32, f32);
}

pub trait Vec4 {
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn z(&self) -> f32;
    fn w(&self) -> f32;

    fn set_x(&mut self, x: f32);
    fn set_y(&mut self, y: f32);
    fn set_z(&mut self, z: f32);
    fn set_w(&mut self, w: f32);

    fn set_xy(&mut self, x: f32, y: f32);
    fn set_xz(&mut self, x: f32, z: f32);
    fn set_xw(&mut self, x: f32, w: f32);
    fn set_yz(&mut self, y: f32, z: f32);
    fn set_yw(&mut self, y: f32, w: f32);
    fn set_zw(&mut self, z: f32, w: f32);

    fn set_xyz(&mut self, x: f32, y: f32, z: f32);
    fn set_xyw(&mut self, x: f32, y: f32, w: f32);
    fn set_xzw(&mut self, x: f32, z: f32, w: f32);
    fn set_yzw(&mut self, y: f32, z: f32, w: f32);

    fn set_xyzw(&mut self, x: f32, y: f32, z: f32, w: f32);

    fn zero() -> Self;
    fn one() -> Self;
    fn unit_x() -> Self;
    fn unit_y() -> Self;
    fn unit_z() -> Self;
    fn unit_w() -> Self;
    fn magnitude(&self) -> f32;
    fn normalize(&mut self);
    fn normalized(&self) -> Self;
    fn dot(&self, other: &Self) -> f32;
    fn distance(&self, other: &Self) -> f32;
    fn lerp(&self, other: &Self, t: f32) -> Self;
}

pub trait Mat2 {
    type Vec2: Vec2;

    fn m11(&self) -> f32;
    fn m12(&self) -> f32;
    fn m21(&self) -> f32;
    fn m22(&self) -> f32;

    fn col1(&self) -> Self::Vec2;
    fn col2(&self) -> Self::Vec2;

    fn set_col1(&mut self, col: &Self::Vec2);
    fn set_col2(&mut self, col: &Self::Vec2);
    fn set_cols(&mut self, col1: &Self::Vec2, col2: &Self::Vec2);

    fn identity() -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn determinant(&self) -> f32;
    fn transpose(&mut self);
    fn transposed(&self) -> Self;
    fn invert(&mut self);
    fn inverted(&self) -> Self;

    fn to_array(&self) -> [f32; 4];
    fn to_tuple(&self) -> (f32, f32, f32, f32);
    fn to_cols(&self) -> (Self::Vec2, Self::Vec2);
    fn from_slice(slice: &[f32]) -> Self;
    fn from_tuple(t: (f32, f32, f32, f32)) -> Self;
    fn from_cols(col1: &Self::Vec2, col2: &Self::Vec2) -> Self;
}

pub trait Mat3 {
    type Vec3: Vec3;

    fn m11(&self) -> f32;
    fn m12(&self) -> f32;
    fn m13(&self) -> f32;
    fn m21(&self) -> f32;
    fn m22(&self) -> f32;
    fn m23(&self) -> f32;
    fn m31(&self) -> f32;
    fn m32(&self) -> f32;
    fn m33(&self) -> f32;

    fn col1(&self) -> Self::Vec3;
    fn col2(&self) -> Self::Vec3;
    fn col3(&self) -> Self::Vec3;

    fn set_col1(&mut self, col: &Self::Vec3);
    fn set_col2(&mut self, col: &Self::Vec3);
    fn set_col3(&mut self, col: &Self::Vec3);
    fn set_cols(&mut self, col1: &Self::Vec3, col2: &Self::Vec3, col3: &Self::Vec3);

    fn identity() -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn determinant(&self) -> f32;
    fn transpose(&mut self);
    fn transposed(&self) -> Self;
    fn invert(&mut self);
    fn inverted(&self) -> Self;

    fn to_array(&self) -> [f32; 9];
    fn to_tuple(&self) -> (f32, f32, f32, f32, f32, f32, f32, f32, f32);
    fn to_cols(&self) -> (Self::Vec3, Self::Vec3, Self::Vec3);
    fn from_slice(slice: &[f32]) -> Self;
    fn from_tuple(t: (f32, f32, f32, f32, f32, f32, f32, f32, f32)) -> Self;
    fn from_cols(col1: &Self::Vec3, col2: &Self::Vec3, col3: &Self::Vec3) -> Self;
}

pub trait Mat4 {
    type Vec4: Vec4;

    fn m11(&self) -> f32;
    fn m12(&self) -> f32;
    fn m13(&self) -> f32;
    fn m14(&self) -> f32;
    fn m21(&self) -> f32;
    fn m22(&self) -> f32;
    fn m23(&self) -> f32;
    fn m24(&self) -> f32;
    fn m31(&self) -> f32;
    fn m32(&self) -> f32;
    fn m33(&self) -> f32;
    fn m34(&self) -> f32;
    fn m41(&self) -> f32;
    fn m42(&self) -> f32;
    fn m43(&self) -> f32;
    fn m44(&self) -> f32;

    fn col1(&self) -> Self::Vec4;
    fn col2(&self) -> Self::Vec4;
    fn col3(&self) -> Self::Vec4;
    fn col4(&self) -> Self::Vec4;

    fn set_col1(&mut self, col: &Self::Vec4);
    fn set_col2(&mut self, col: &Self::Vec4);
    fn set_col3(&mut self, col: &Self::Vec4);
    fn set_col4(&mut self, col: &Self::Vec4);
    fn set_cols(
        &mut self,
        col1: &Self::Vec4,
        col2: &Self::Vec4,
        col3: &Self::Vec4,
        col4: &Self::Vec4,
    );

    fn identity() -> Self;
    fn zero() -> Self;
    fn one() -> Self;
    fn determinant(&self) -> f32;
    fn transpose(&mut self);
    fn transposed(&self) -> Self;
    fn invert(&mut self);
    fn inverted(&self) -> Self;

    fn to_array(&self) -> [f32; 16];
    fn to_cols(&self) -> (Self::Vec4, Self::Vec4, Self::Vec4, Self::Vec4);
    fn from_slice(slice: &[f32]) -> Self;
    fn from_cols(
        col1: &Self::Vec4,
        col2: &Self::Vec4,
        col3: &Self::Vec4,
        col4: &Self::Vec4,
    ) -> Self;
}
