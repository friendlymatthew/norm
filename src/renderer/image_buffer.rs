use crate::renderer::TextureVertex;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{Buffer, BufferUsages, Device};

#[derive(Debug)]
pub(crate) struct ImageBuffer {
    vertex_buffer: Buffer,
    index_buffer: Buffer,
}

impl ImageBuffer {
    const VERTICES: [TextureVertex; 4] = [
        TextureVertex {
            position: [-1.0, -1.0, 0.0],
            tex_coords: [0.0, 1.0],
        },
        TextureVertex {
            position: [-1.0, 1.0, 0.0],
            tex_coords: [0.0, 0.0],
        },
        TextureVertex {
            position: [1.0, -1.0, 0.0],
            tex_coords: [1.0, 1.0],
        },
        TextureVertex {
            position: [1.0, 1.0, 0.0],
            tex_coords: [1.0, 0.0],
        },
    ];

    const INDICES: [u16; 6] = [
        0, 1, 2, // first triangle
        2, 1, 3, // second triangle
    ];

    pub(crate) fn new(device: &Device) -> Self {
        Self {
            vertex_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Image TextureVertex Buffer"),
                contents: bytemuck::cast_slice(&Self::VERTICES),
                usage: BufferUsages::VERTEX,
            }),
            index_buffer: device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Index Buffer"),
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
