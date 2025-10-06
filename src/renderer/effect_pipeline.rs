use super::{compute_effect::ComputeEffect, gpu_state::GpuResourceAllocator, Texture};
use anyhow::Result;

use crate::renderer::feature_uniform::FeatureUniform;

pub struct EffectPipeline {
    effects: Vec<(ComputeEffect, Box<dyn Fn(&FeatureUniform) -> bool>)>,

    texture_a: Texture,
    texture_b: Texture,
}

impl std::fmt::Debug for EffectPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectPipeline")
            .field("effects_count", &self.effects.len())
            .field("texture_a", &self.texture_a)
            .field("texture_b", &self.texture_b)
            .finish()
    }
}

impl EffectPipeline {
    pub fn builder<'a, 'b>(
        gpu_allocator: &'a GpuResourceAllocator<'b>,
        input_texture: &'a wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> EffectPipelineBuilder<'a, 'b> {
        EffectPipelineBuilder::new(gpu_allocator, input_texture, width, height)
    }

    pub fn texture_a(&self) -> &Texture {
        &self.texture_a
    }

    pub fn execute(
        &self,
        feature_uniform: &FeatureUniform,
        encoder: &mut wgpu::CommandEncoder,
        workgroup_count_x: u32,
        workgroup_count_y: u32,
    ) {
        for (effect, condition) in &self.effects {
            if condition(feature_uniform) {
                effect.dispatch(encoder, workgroup_count_x, workgroup_count_y);
            }
        }
    }

    pub fn update_effect_uniform<T: bytemuck::Pod>(
        &self,
        queue: &wgpu::Queue,
        index: usize,
        data: T,
    ) {
        if let Some((effect, _)) = self.effects.get(index) {
            effect.update_uniform(queue, data);
        }
    }
}

pub struct EffectPipelineBuilder<'a, 'b> {
    gpu_allocator: &'a GpuResourceAllocator<'b>,
    input_texture: &'a wgpu::TextureView,
    texture_a: Texture,
    texture_b: Texture,
    effects: Vec<(ComputeEffect, Box<dyn Fn(&FeatureUniform) -> bool>)>,
}

impl<'a, 'b> EffectPipelineBuilder<'a, 'b> {
    fn new(
        gpu_allocator: &'a GpuResourceAllocator<'b>,
        input_texture: &'a wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Self {
        let texture_a = gpu_allocator.create_storage_texture("pipeline_texture_a", width, height);
        let texture_b = gpu_allocator.create_storage_texture("pipeline_texture_b", width, height);

        Self {
            gpu_allocator,
            input_texture,
            texture_a,
            texture_b,
            effects: Vec::new(),
        }
    }

    pub fn add_effect<U, F>(
        mut self,
        label: &str,
        shader_source: &str,
        uniform: Option<U>,
        condition: F,
    ) -> Result<Self>
    where
        U: bytemuck::Pod,
        F: Fn(&FeatureUniform) -> bool + 'static,
    {
        let effect_count = self.effects.len();

        let (input_view, output_view) = if effect_count == 0 {
            (self.input_texture, &self.texture_a.view)
        } else if effect_count % 2 == 1 {
            (&self.texture_a.view, &self.texture_b.view)
        } else {
            (&self.texture_b.view, &self.texture_a.view)
        };

        let mut builder = ComputeEffect::builder(label).with_shader(shader_source);

        if let Some(data) = uniform {
            builder = builder.with_uniform(data);
        }

        let effect = builder.build(&self.gpu_allocator.device, input_view, output_view)?;
        self.effects.push((effect, Box::new(condition)));

        Ok(self)
    }

    pub fn build(self) -> Result<EffectPipeline> {
        Ok(EffectPipeline {
            effects: self.effects,
            texture_a: self.texture_a,
            texture_b: self.texture_b,
        })
    }
}
