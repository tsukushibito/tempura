use ash::vk;

pub struct Mesh {}
pub struct Material {}
pub struct Object {
    meshes: Vec<Mesh>,
    materials: Vec<Material>,
}
pub struct Light {}
pub struct Camera {}

pub struct Scene {
    objects: Vec<Object>,
    lights: Vec<Light>,
    cameras: Vec<Camera>,
}
