use crate::renderer::Vertex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, Device};

pub(crate) struct RectangleBuffer {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl RectangleBuffer {
    const WHITE: [f32; 3] = [0.0, 0.0, 0.0];

    const VERTICES: [Vertex; 4] = [
        Vertex {
            position: [0.6, -1.0, 0.0],
            color: Self::WHITE,
        },
        Vertex {
            position: [1.0, -1.0, 0.0],
            color: Self::WHITE,
        },
        Vertex {
            position: [1.0, 1.0, 0.0],
            color: Self::WHITE,
        },
        Vertex {
            position: [0.6, 1.0, 0.0],
            color: Self::WHITE,
        },
    ];

    const INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

    pub(crate) fn new(device: &Device) -> Self {
        Self {
            vertex_buffer: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Rectangle Vertex Buffer"),
                contents: bytemuck::cast_slice(&Self::VERTICES),
                usage: BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Rectangle Index Buffer"),
                contents: bytemuck::cast_slice(&Self::INDICES),
                usage: BufferUsages::INDEX,
            }),
        }
    }

    pub(crate) fn vertex_buffer(&self) -> &Buffer {
        &self.vertex_buffer
    }

    pub(crate) fn index_buffer(&self) -> &Buffer {
        &self.index_buffer
    }
    pub(crate) fn num_indices(&self) -> u32 {
        Self::INDICES.len() as u32
    }
}
