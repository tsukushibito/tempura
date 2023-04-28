pub enum PassType {
    Graphics,
    Compute,
    ComputeAsync,
    Copy,
}

pub enum PassFlags {
    None = 0x00,
    ForceNoCull = 0x01,
    AllowUAVWrites = 0x02,
    SkipAutoRenderPass = 0x04,
    LegacyRenderPass = 0x08,
    ActAsCreatorWhenWriting = 0x10,
}

pub enum ReadAccess {
    PixelShader,
    NonPixelShader,
    AllShader,
}

pub enum LoadAccessOp {
    Discard,
    Preserve,
    Clear,
    NoAccess,
}

pub enum StoreAccessOp {
    Discard,
    Preserve,
    Resolve,
    NoAccess,
}
struct Pass {}
