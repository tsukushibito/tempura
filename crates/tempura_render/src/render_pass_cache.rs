use std::{
    cell::RefCell,
    collections::{hash_map::DefaultHasher, HashMap},
    hash::{Hash, Hasher},
    rc::Rc,
};

use ash::vk;
use tempura_vulkan::{Device, RenderPass};

pub(crate) struct RenderPassCache {
    render_passes: RefCell<HashMap<u64, Rc<RenderPass>>>,
}

impl RenderPassCache {
    pub(crate) fn new() -> Self {
        Self {
            render_passes: RefCell::new(HashMap::new()),
        }
    }

    pub(crate) fn get_or_create(
        &self,
        device: &Rc<Device>,
        attachments: &[vk::AttachmentDescription],
        subpasses: &[vk::SubpassDescription],
        dependencies: &[vk::SubpassDependency],
    ) -> (Rc<RenderPass>, bool) {
        let mut hasher = DefaultHasher::new();
        attachments.iter().for_each(|a| {
            let a = VkAttachmentDescription(*a);
            a.hash(&mut hasher);
        });
        subpasses.iter().for_each(|s| {
            let s = VkSubpassDescription(*s);
            s.hash(&mut hasher);
        });
        dependencies.iter().for_each(|d| {
            let d = VkSubpassDependency(*d);
            d.hash(&mut hasher);
        });
        let hash = hasher.finish();

        let mut render_passes = self.render_passes.borrow_mut();
        if let Some(render_pass) = render_passes.get(&hash) {
            return (render_pass.clone(), false);
        }

        let render_pass = Rc::new(
            RenderPass::new(device, attachments, subpasses, dependencies)
                .expect("Failed to create render pass"),
        );
        render_passes.insert(hash, render_pass.clone());
        (render_pass, true)
    }
}

struct VkAttachmentDescription(vk::AttachmentDescription);

impl Hash for VkAttachmentDescription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.flags.hash(state);
        self.0.format.hash(state);
        self.0.samples.hash(state);
        self.0.load_op.hash(state);
        self.0.store_op.hash(state);
        self.0.stencil_load_op.hash(state);
        self.0.stencil_store_op.hash(state);
        self.0.initial_layout.hash(state);
        self.0.final_layout.hash(state);
    }
}

struct VkSubpassDescription(vk::SubpassDescription);

impl Hash for VkSubpassDescription {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.flags.hash(state);
        self.0.pipeline_bind_point.hash(state);
        self.0.input_attachment_count.hash(state);
        self.0.p_input_attachments.hash(state);
        self.0.color_attachment_count.hash(state);
        self.0.p_color_attachments.hash(state);
        self.0.p_resolve_attachments.hash(state);
        self.0.p_depth_stencil_attachment.hash(state);
        self.0.preserve_attachment_count.hash(state);
        self.0.p_preserve_attachments.hash(state);
    }
}

struct VkSubpassDependency(vk::SubpassDependency);

impl Hash for VkSubpassDependency {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.src_subpass.hash(state);
        self.0.dst_subpass.hash(state);
        self.0.src_stage_mask.hash(state);
        self.0.dst_stage_mask.hash(state);
        self.0.src_access_mask.hash(state);
        self.0.dst_access_mask.hash(state);
        self.0.dependency_flags.hash(state);
    }
}
