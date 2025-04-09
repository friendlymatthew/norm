use crate::renderer::Texture;
use wgpu::{
    BindGroup,
    BindGroupLayout,
};

#[derive(Debug)]
pub struct ShaderResourceInner<T> {
    pub resource: T,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

pub type TextureResource = ShaderResourceInner<Texture>;
pub type UniformResource = ShaderResourceInner<wgpu::Buffer>;

#[derive(Debug)]
pub struct Shader {
    // pub pipeline_layout: wgpu::PipelineLayout,
    pub render_pipeline: wgpu::RenderPipeline,

    pub texture_resources: Vec<TextureResource>,
    pub uniform_resources: Vec<UniformResource>,
}
