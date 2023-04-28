pub enum ResourceType {
    Buffer,
    Texture,
}

pub enum ResourceMode {
    CopySrc,
    CopyDst,
    IndirectArgs,
    Vertex,
    Index,
    Constant,
}

pub enum DescriptorType {
    ReadOnly,
    ReadWrite,
    RenderTarget,
    DepthStencil,
}

struct BufferId(u64);
struct TextureId(u64);
