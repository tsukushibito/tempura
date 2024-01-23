use std::{
    cell::Cell,
    collections::{HashMap, VecDeque},
    error::Error,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Default, Debug)]
struct IdGenerator {
    next: Cell<usize>,
}

impl IdGenerator {
    pub fn new() -> Self {
        IdGenerator { next: Cell::new(0) }
    }

    pub fn generate(&self) -> usize {
        let id = self.next.get();
        self.next.set(id + 1);
        id
    }
}

pub struct Texture {
    // テクスチャのデータや状態を保持するフィールド
}

pub struct TextureDesc {}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct TextureHandle(usize);

struct ResourceManager {
    textures: HashMap<TextureHandle, Texture>,
    texture_descs: HashMap<TextureHandle, TextureDesc>,
    id_generator: IdGenerator,
}

impl ResourceManager {
    pub fn create_texture(&mut self, desc: TextureDesc) -> TextureHandle {
        let handle = TextureHandle(self.id_generator.generate());
        self.texture_descs.insert(handle, desc);
        handle
    }

    pub fn get_texture(&self, handle: &TextureHandle) -> Option<&Texture> {
        self.textures.get(handle)
    }

    pub fn release_texture(&mut self, handle: &TextureHandle) {
        self.textures.remove(handle);
        self.texture_descs.remove(handle);
    }
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct RenderPassHandle(usize);

trait RenderPass {
    fn execute(&self, resource_manager: &ResourceManager);
    fn read_texture_handles(&self) -> &[TextureHandle];
    fn write_texture_handles(&self) -> &[TextureHandle];
}

#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug, Default)]
struct Edge {
    from: Option<RenderPassHandle>,
    to: Option<RenderPassHandle>,
}

struct RenderGraph {
    resource_manager: ResourceManager,
    render_passes: HashMap<RenderPassHandle, Box<dyn RenderPass>>,
    id_generator: IdGenerator,
}

impl RenderGraph {
    fn new() -> Self {
        let resource_manager = ResourceManager {
            textures: Default::default(),
            texture_descs: Default::default(),
            id_generator: Default::default(),
        };

        RenderGraph {
            resource_manager,
            render_passes: Default::default(),
            id_generator: Default::default(),
        }
    }

    fn create_texture(&mut self, desc: TextureDesc) -> TextureHandle {
        self.resource_manager.create_texture(desc)
    }

    pub fn add_render_pass<T: RenderPass + 'static>(&mut self, render_pass: T) -> RenderPassHandle {
        let handle = RenderPassHandle(self.id_generator.generate());
        self.render_passes.insert(handle, Box::new(render_pass));
        handle
    }

    pub fn execute(&mut self) {
        let render_pass_handles = self.topological_sort().unwrap();
        for handle in render_pass_handles {
            self.render_passes[&handle].execute(&self.resource_manager);
        }
    }

    fn topological_sort(&self) -> Result<Vec<RenderPassHandle>> {
        // グラフ構築
        // テクスチャをキーとして読み書きするパスを記録
        let mut writers = HashMap::<TextureHandle, Vec<RenderPassHandle>>::new();
        let mut readers = HashMap::<TextureHandle, Vec<RenderPassHandle>>::new();
        for (ph, p) in &self.render_passes {
            for th in p.read_texture_handles() {
                readers.entry(*th).or_insert(Default::default()).push(*ph);
            }
            for th in p.write_texture_handles() {
                writers.entry(*th).or_insert(Default::default()).push(*ph);
            }
        }
        // 記録した内容を元にグラフを構築するとともに、入次数を計算
        let mut graph = HashMap::<RenderPassHandle, Vec<RenderPassHandle>>::new();
        let mut in_degrees: HashMap<RenderPassHandle, usize> = HashMap::new();
        for (rh, _) in &self.render_passes {
            graph.insert(*rh, Default::default());
            in_degrees.insert(*rh, 0);
        }
        for (th, froms) in &writers {
            if let Some(tos) = readers.get(th) {
                for to in tos {
                    let in_degree = in_degrees.entry(*to).or_default();
                    *in_degree += froms.len();
                }
                for from in froms {
                    graph.get_mut(from).unwrap().extend(tos);
                }
            }
        }

        let mut queue: VecDeque<RenderPassHandle> = VecDeque::new();
        let mut result: Vec<RenderPassHandle> = Vec::new();

        for (&render_pass_handle, &degree) in &in_degrees {
            if degree == 0 {
                queue.push_back(render_pass_handle);
            }
        }

        while let Some(handle) = queue.pop_front() {
            for to in &graph[&handle] {
                let in_degree = in_degrees.get_mut(&to).unwrap();
                *in_degree -= 1;
                if *in_degree == 0 {
                    queue.push_back(*to);
                }
            }
            result.push(handle);
        }

        // 全てのノードを処理したかチェック（サイクルの存在チェック）
        if result.len() != self.render_passes.len() {
            panic!();
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use super::*;

    struct TestRenderPass {
        id: usize,
        inputs: Vec<TextureHandle>,
        outputs: Vec<TextureHandle>,
        pub executed_queue: Rc<RefCell<VecDeque<usize>>>,
    }

    impl RenderPass for TestRenderPass {
        fn execute(&self, resource_manager: &ResourceManager) {
            // let input_tex = resource_manager.get_texture(&self.input);
            // let output_tex = resource_manager.get_texture(&self.output);
            self.executed_queue.borrow_mut().push_back(self.id);
        }

        fn read_texture_handles(&self) -> &[TextureHandle] {
            &self.inputs
        }

        fn write_texture_handles(&self) -> &[TextureHandle] {
            &self.outputs
        }
    }

    #[test]
    fn test_render_graph() {
        let mut render_graph = RenderGraph::new();

        let executed_queue: Rc<RefCell<VecDeque<usize>>> = Default::default();

        let desc0 = TextureDesc {};
        let tex0 = render_graph.create_texture(desc0);

        let desc1 = TextureDesc {};
        let tex1 = render_graph.create_texture(desc1);

        let desc2 = TextureDesc {};
        let tex2 = render_graph.create_texture(desc2);

        let desc3 = TextureDesc {};
        let tex3 = render_graph.create_texture(desc3);

        let render_pass_0 = TestRenderPass {
            id: 0,
            inputs: Default::default(),
            outputs: vec![tex0],
            executed_queue: executed_queue.clone(),
        };
        render_graph.add_render_pass(render_pass_0);

        let render_pass_1 = TestRenderPass {
            id: 1,
            inputs: vec![tex0],
            outputs: vec![tex1],
            executed_queue: executed_queue.clone(),
        };
        render_graph.add_render_pass(render_pass_1);

        let render_pass_2 = TestRenderPass {
            id: 2,
            inputs: vec![tex0],
            outputs: vec![tex2],
            executed_queue: executed_queue.clone(),
        };
        render_graph.add_render_pass(render_pass_2);

        let render_pass_3 = TestRenderPass {
            id: 3,
            inputs: vec![tex1, tex2],
            outputs: vec![tex3],
            executed_queue: executed_queue.clone(),
        };
        render_graph.add_render_pass(render_pass_3);

        render_graph.execute();

        let id_0 = executed_queue.borrow_mut().pop_front().unwrap();
        let id_1 = executed_queue.borrow_mut().pop_front().unwrap();
        let id_2 = executed_queue.borrow_mut().pop_front().unwrap();
        let id_3 = executed_queue.borrow_mut().pop_front().unwrap();

        assert_eq!(id_0, 0);
        assert!(id_1 == 1 || id_1 == 2);
        assert!(id_2 == 1 || id_2 == 2);
        assert_eq!(id_3, 3);
    }
}
