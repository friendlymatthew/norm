use crate::png::grammar::Png;
use crate::renderer::draw_uniform::DrawUniform;
use crate::renderer::feature_uniform::FeatureUniform;
use crate::renderer::{Texture, Vertex};
use anyhow::{anyhow, Result};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Backends, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, BlendComponent,
    BlendState, Buffer, BufferBindingType, BufferUsages, ColorTargetState, ColorWrites,
    FragmentState, FrontFace, MultisampleState, PipelineLayout, PipelineLayoutDescriptor,
    PolygonMode, PrimitiveState, PrimitiveTopology, RenderPipeline, RenderPipelineDescriptor,
    SamplerBindingType, ShaderModule, ShaderModuleDescriptor, ShaderSource, ShaderStages,
    TextureSampleType, TextureViewDimension, VertexState,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

#[derive(Debug)]
pub struct Shader {
    pub module: ShaderModule,
    pub pipeline_layout: PipelineLayout,
    pub render_pipeline: RenderPipeline,
    pub resources: [ShaderResource; 3], // todo, fix this
}

#[derive(Debug)]
pub enum UniformBufferType {
    Feature(FeatureUniform),
    Draw(DrawUniform),
}

#[derive(Debug)]
pub enum ShaderResourceType {
    Texture(Texture),
    Uniform(Buffer, UniformBufferType),
}

#[derive(Debug)]
pub struct ShaderResource {
    pub resource: ShaderResourceType,
    pub bind_group: BindGroup,
    pub bind_group_layout: BindGroupLayout,
}

#[derive(Debug)]
pub struct GPUDevice<'a> {
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
    pub(crate) surface: wgpu::Surface<'a>,
    pub(crate) surface_configuration: wgpu::SurfaceConfiguration,
    pub(crate) size: PhysicalSize<u32>,
}

impl<'a> GPUDevice<'a> {
    pub async fn new(window: &'a Window) -> Result<Self> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            #[cfg(not(target_arch = "wasm32"))]
            backends: Backends::PRIMARY,
            #[cfg(target_arch = "wasm32")]
            backends: Backends::GL,
            ..Default::default()
        });

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("Failed to get adapter"))?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    memory_hints: Default::default(),
                },
                None, // Trace path
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let surface_configuration = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        Ok(Self {
            device,
            queue,
            size,
            surface,
            surface_configuration,
        })
    }

    pub(crate) fn create_texture(&self, id: &str, png: &Png) -> Result<ShaderResource> {
        let diffuse_texture = Texture::from_bytes(&self.device, &self.queue, png)?;

        let texture_bind_group_layout =
            self.device
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    entries: &[
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Texture {
                                multisampled: false,
                                view_dimension: TextureViewDimension::D2,
                                sample_type: TextureSampleType::Float { filterable: true },
                            },
                            count: None,
                        },
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::FRAGMENT,
                            ty: BindingType::Sampler(SamplerBindingType::Filtering),
                            count: None,
                        },
                    ],
                    label: Some(id),
                });

        let diffuse_bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
            label: Some(id),
        });

        Ok(ShaderResource {
            resource: ShaderResourceType::Texture(diffuse_texture),
            bind_group: diffuse_bind_group,
            bind_group_layout: texture_bind_group_layout,
        })
    }

    pub(crate) fn surface_dimension(&self) -> (u32, u32) {
        (
            self.surface_configuration.width,
            self.surface_configuration.height,
        )
    }

    pub(crate) fn create_uniform<T>(
        &self,
        id: &str,
        uniform: UniformBufferType,
    ) -> Result<ShaderResource> {
        let buffer = match uniform {
            UniformBufferType::Draw(d) => self.device.create_buffer_init(&BufferInitDescriptor {
                label: Some(id),
                contents: bytemuck::cast_slice(&[d]),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            }),
            UniformBufferType::Feature(f) => {
                self.device.create_buffer_init(&BufferInitDescriptor {
                    label: Some(id),
                    contents: bytemuck::cast_slice(&[f]),
                    usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                })
            }
        };

        let bind_group_layout = self
            .device
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX_FRAGMENT,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some(id),
            });

        let bind_group = self.device.create_bind_group(&BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
            label: Some(id),
        });

        Ok(ShaderResource {
            resource: ShaderResourceType::Uniform(buffer, uniform),
            bind_group,
            bind_group_layout,
        })
    }

    pub(crate) fn create_shader(
        &self,
        id: &str,
        shader_file: &str,
        resources: [ShaderResource; 3],
    ) -> Result<Shader> {
        let bind_group_layouts = &resources
            .iter()
            .map(|r| &r.bind_group_layout)
            .collect::<Vec<_>>();

        let module = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(id),
            source: ShaderSource::Wgsl(shader_file.into()),
        });

        let pipeline_layout = self
            .device
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some(id),
                bind_group_layouts: &bind_group_layouts,
                push_constant_ranges: &[],
            });

        let render_pipeline = self
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(id),
                layout: Some(&pipeline_layout),
                vertex: VertexState {
                    module: &module,
                    entry_point: "vs_main",
                    buffers: &[Vertex::desc()],
                    compilation_options: Default::default(),
                },
                fragment: Some(FragmentState {
                    module: &module,
                    entry_point: "fs_main",
                    targets: &[Some(ColorTargetState {
                        format: self.surface_configuration.format,
                        blend: Some(BlendState {
                            color: BlendComponent::REPLACE,
                            alpha: BlendComponent::REPLACE,
                        }),
                        write_mask: ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                primitive: PrimitiveState {
                    topology: PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: FrontFace::Ccw,
                    cull_mode: None,
                    // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                    // or Features::POLYGON_MODE_POINT
                    polygon_mode: PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview renderer pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
                // Useful for optimizing shader compilation on Android
                cache: None,
            });

        Ok(Shader {
            module,
            pipeline_layout,
            render_pipeline,
            resources,
        })
    }

    pub(crate) fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.size = new_size;
        self.surface_configuration.width = new_size.width;
        self.surface_configuration.height = new_size.height;
        self.surface
            .configure(&self.device, &self.surface_configuration);
    }

    pub(crate) fn update_uniform(&self, image_shader: &Shader) {
        if let ShaderResourceType::Uniform(
            feature_buffer,
            UniformBufferType::Feature(feature_uniform),
        ) = &image_shader.resources[1].resource
        {
            self.queue
                .write_buffer(feature_buffer, 0, bytemuck::cast_slice(&[*feature_uniform]));
        }

        if let ShaderResourceType::Uniform(draw_buffer, UniformBufferType::Draw(draw_uniform)) =
            &image_shader.resources[2].resource
        {
            self.queue
                .write_buffer(draw_buffer, 0, bytemuck::cast_slice(&[*draw_uniform]));
        }
    }
}
