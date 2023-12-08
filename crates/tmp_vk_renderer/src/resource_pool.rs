use ash::vk;
use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::{Hash, Hasher};
use std::sync::Mutex;

pub(crate) struct ResourcePool<K, R> {
    resources: Mutex<HashMap<K, R>>,
    create_fn: Box<dyn Fn(&K) -> R + Send + Sync>,
    destroy_fn: Box<dyn Fn(&R) + Send + Sync>,
}

impl<K, R> ResourcePool<K, R>
where
    K: Eq + Hash + Clone,
    R: Clone,
{
    pub(crate) fn new<F, D>(create_fn: F, destroy_fn: D) -> Self
    where
        F: Fn(&K) -> R + Send + Sync + 'static,
        D: Fn(&R) + Send + Sync + 'static,
    {
        ResourcePool {
            resources: Mutex::new(HashMap::new()),
            create_fn: Box::new(create_fn),
            destroy_fn: Box::new(destroy_fn),
        }
    }

    pub(crate) fn get(&self, key: &K) -> R {
        let mut resources = self.resources.lock().unwrap();
        resources
            .entry(key.clone())
            .or_insert_with(|| (self.create_fn)(key))
            .clone()
    }

    pub(crate) fn release(&self, key: &K) {
        let mut resources = self.resources.lock().unwrap();
        if let Some(resource) = resources.remove(key) {
            (self.destroy_fn)(&resource);
        }
    }
}

impl<K, R> Drop for ResourcePool<K, R> {
    fn drop(&mut self) {
        let resources = self.resources.lock().unwrap();
        for resource in resources.values() {
            (self.destroy_fn)(resource);
        }
    }
}

pub(crate) fn generate_key_hash<K: Hash>(key: &K) -> u64 {
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish()
}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub(crate) struct FramebufferKey {
    image_views: Vec<vk::ImageView>,
    render_pass: vk::RenderPass,
    width: u32,
    height: u32,
}

impl FramebufferKey {
    pub(crate) fn new(
        image_views: Vec<vk::ImageView>,
        render_pass: vk::RenderPass,
        width: u32,
        height: u32,
    ) -> FramebufferKey {
        FramebufferKey {
            image_views,
            render_pass,
            width,
            height,
        }
    }
}

pub(crate) type FramebufferPool = ResourcePool<FramebufferKey, vk::Framebuffer>;
