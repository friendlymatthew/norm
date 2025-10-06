use anyhow::Result;
use wgpu::util::DeviceExt;

// a compute shader effect that can be applied to textures
#[derive(Debug)]
pub struct ComputeEffect {
    pub pipeline: wgpu::ComputePipeline,
    pub bind_group: wgpu::BindGroup,
    uniform_buffer: Option<wgpu::Buffer>,
}

impl ComputeEffect {
    pub const fn builder<'a>(label: &'a str) -> ComputeEffectBuilder<'a> {
        ComputeEffectBuilder {
            label,
            shader_source: None,
            entry_point: "main",
            uniform_data: None,
        }
    }

    pub fn update_uniform<T: bytemuck::Pod>(&self, queue: &wgpu::Queue, data: T) {
        if let Some(buffer) = &self.uniform_buffer {
            queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[data]));
        }
    }

    pub fn dispatch(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        workgroup_count_x: u32,
        workgroup_count_y: u32,
    ) {
        let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });

        compute_pass.set_pipeline(&self.pipeline);
        compute_pass.set_bind_group(0, &self.bind_group, &[]);
        compute_pass.dispatch_workgroups(workgroup_count_x, workgroup_count_y, 1);
    }
}

pub struct ComputeEffectBuilder<'a> {
    label: &'a str,
    shader_source: Option<&'a str>,
    entry_point: &'a str,
    uniform_data: Option<Vec<u8>>,
}

impl<'a> ComputeEffectBuilder<'a> {
    pub const fn with_shader(mut self, source: &'a str) -> Self {
        self.shader_source = Some(source);
        self
    }

    pub const fn _with_entry_point(mut self, entry_point: &'a str) -> Self {
        self.entry_point = entry_point;
        self
    }

    pub fn with_uniform<T: bytemuck::Pod>(mut self, data: T) -> Self {
        self.uniform_data = Some(bytemuck::cast_slice(&[data]).to_vec());
        self
    }

    pub fn build(
        self,
        device: &wgpu::Device,
        input_texture: &wgpu::TextureView,
        output_texture: &wgpu::TextureView,
    ) -> Result<ComputeEffect> {
        let shader_source = self
            .shader_source
            .ok_or_else(|| anyhow::anyhow!("Shader source is required"))?;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(self.label),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        let uniform_buffer = self.uniform_data.as_ref().map(|data| {
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{}_uniform", self.label)),
                contents: data,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            })
        });

        let mut bind_group_layout_entries = vec![
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Texture {
                    multisampled: false,
                    view_dimension: wgpu::TextureViewDimension::D2,
                    sample_type: wgpu::TextureSampleType::Float { filterable: false },
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::StorageTexture {
                    access: wgpu::StorageTextureAccess::WriteOnly,
                    format: wgpu::TextureFormat::Rgba8Unorm,
                    view_dimension: wgpu::TextureViewDimension::D2,
                },
                count: None,
            },
        ];

        if uniform_buffer.is_some() {
            bind_group_layout_entries.push(wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            });
        }

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &bind_group_layout_entries,
            label: Some(&format!("{}_bind_group_layout", self.label)),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some(&format!("{}_pipeline_layout", self.label)),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some(self.label),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: self.entry_point,
            compilation_options: Default::default(),
            cache: None,
        });

        let mut bind_group_entries = vec![
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(input_texture),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(output_texture),
            },
        ];

        if let Some(buffer) = &uniform_buffer {
            bind_group_entries.push(wgpu::BindGroupEntry {
                binding: 2,
                resource: buffer.as_entire_binding(),
            });
        }

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &bind_group_entries,
            label: Some(&format!("{}_bind_group", self.label)),
        });

        Ok(ComputeEffect {
            pipeline,
            bind_group,
            uniform_buffer,
        })
    }
}
