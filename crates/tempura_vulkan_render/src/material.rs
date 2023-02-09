use std::{ffi::CString, rc::Rc};

use ash::vk;

use crate::Device;

use super::Shader;

pub struct Material {
    shader: Rc<Shader>,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
}

impl Material {
    pub(crate) fn new(device: &Rc<Device>, shader: &Rc<Shader>) -> Self {
        unsafe {
            let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
                .flags(vk::PipelineLayoutCreateFlags::empty())
                .set_layouts(&[])
                .push_constant_ranges(&[])
                .build();
            let pipeline_layout = device
                .device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("create_pipeline_layout failed.");

            let vertex_shader_entry_point = shader.vertex_shader_reflect.get_entry_point_name();
            let vertex_shader_entry_point = CString::new(vertex_shader_entry_point).unwrap();
            let stages = vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(shader.vertex_shader)
                .name(vertex_shader_entry_point.as_c_str())
                .build();
            let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder().build();
            let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                .primitive_restart_enable(false)
                .build();
            let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
                .depth_clamp_enable(false)
                .rasterizer_discard_enable(false)
                .polygon_mode(vk::PolygonMode::FILL)
                .line_width(1.0)
                .cull_mode(vk::CullModeFlags::NONE)
                .front_face(vk::FrontFace::CLOCKWISE)
                .depth_bias_enable(false)
                .depth_bias_constant_factor(0.0)
                .depth_bias_clamp(0.0)
                .depth_bias_slope_factor(0.0)
                .build();
            let multisample_state = vk::PipelineMultisampleStateCreateInfo::builder()
                .sample_shading_enable(false)
                .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                .min_sample_shading(1.0)
                .sample_mask(&[])
                .alpha_to_coverage_enable(false)
                .alpha_to_one_enable(false)
                .build();
            let viewport_state = vk::PipelineViewportStateCreateInfo::builder()
                .viewports(&[vk::Viewport::builder().width(800.0).height(600.0).build()])
                .scissors(&[vk::Rect2D::default()])
                .build();
            let pipeline_color_blend_attachment_state =
                vk::PipelineColorBlendAttachmentState::builder()
                    .color_write_mask(vk::ColorComponentFlags::RGBA)
                    .blend_enable(false)
                    .build();
            let color_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
                .logic_op_enable(false)
                .logic_op(vk::LogicOp::COPY)
                .attachments(&[pipeline_color_blend_attachment_state])
                .build();

            let dynamic_state = vk::PipelineDynamicStateCreateInfo::builder()
                .dynamic_states(&[vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR])
                .build();

            let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&[stages])
                .vertex_input_state(&vertex_input_state)
                .input_assembly_state(&input_assembly_state)
                .viewport_state(&viewport_state)
                .rasterization_state(&rasterization_state)
                .multisample_state(&multisample_state)
                .color_blend_state(&color_blend_state)
                .layout(pipeline_layout)
                .render_pass(vk::RenderPass::null())
                .dynamic_state(&dynamic_state)
                .subpass(0)
                .build();
            let pipeline = device
                .device
                .create_graphics_pipelines(vk::PipelineCache::null(), &[pipeline_info], None)
                .expect("create_graphics_pipeline failed.");

            Material {
                shader: shader.clone(),
                pipeline: pipeline[0],
                pipeline_layout,
            }
        }
    }
}
