use std::{io::Cursor, rc::Rc};

use ash::{util::read_spv, vk};
use spirv_reflect::ShaderModule;
use tempura_render as tr;

use crate::Device;

pub struct Shader {
    device: Rc<Device>,
    pub(crate) vertex_shader: vk::ShaderModule,
    pub(crate) vertex_shader_reflect: spirv_reflect::ShaderModule,
    pub(crate) fragment_shader: vk::ShaderModule,
    pub(crate) fragment_shader_reflect: spirv_reflect::ShaderModule,
}

impl Shader {
    pub(crate) fn new(
        device: &std::rc::Rc<Device>,
        vertex_shader_code: &Vec<u8>,
        fragment_shader_code: &Vec<u8>,
    ) -> Self {
        unsafe {
            let mut vertex_shader_code = Cursor::new(vertex_shader_code.to_owned());
            let vertex_shader_code = read_spv(&mut vertex_shader_code).expect("read_spv failed.");
            let vertex_shader_create_info = vk::ShaderModuleCreateInfo::builder()
                .code(&vertex_shader_code)
                .build();
            let vertex_shader = device
                .device
                .create_shader_module(&vertex_shader_create_info, None)
                .expect("create_shader_module failed.");
            let vertex_shader_reflect = ShaderModule::load_u32_data(&vertex_shader_code)
                .expect("vertex shader reflection failed.");

            let mut fragment_shader_code = Cursor::new(fragment_shader_code.to_owned());
            let fragment_shader_code =
                read_spv(&mut fragment_shader_code).expect("read_spv failed.");
            let fragment_shader_create_info = vk::ShaderModuleCreateInfo::builder()
                .code(&fragment_shader_code)
                .build();
            let fragment_shader = device
                .device
                .create_shader_module(&fragment_shader_create_info, None)
                .expect("create_shader_module failed.");
            let fragment_shader_reflect = ShaderModule::load_u32_data(&fragment_shader_code)
                .expect("fragment shader reflection failed.");

            Shader {
                device: device.clone(),
                vertex_shader,
                vertex_shader_reflect,
                fragment_shader,
                fragment_shader_reflect,
            }
        }
    }
}

impl Drop for Shader {
    fn drop(&mut self) {
        unsafe {
            self.device
                .device
                .destroy_shader_module(self.vertex_shader, None);
            self.device
                .device
                .destroy_shader_module(self.fragment_shader, None);
        }
    }
}
