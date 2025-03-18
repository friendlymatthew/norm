use super::draw_uniform::DrawUniform;
use super::shape::{compute_radius, Shape, ShapeStack};
use crate::renderer::device::{GPUDevice, Shader, ShaderResourceType, UniformBufferType};
use crate::renderer::feature_uniform::{FeatureUniform, TransformAction};
use crate::renderer::mouse_state::MouseState;
use crate::{png::grammar::Png, renderer::Vertex};
use anyhow::Result;
use std::iter;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    Buffer, BufferUsages, Color, CommandEncoderDescriptor, IndexFormat, LoadOp, Operations,
    RenderPassColorAttachment, RenderPassDescriptor, StoreOp, SurfaceError, TextureViewDescriptor,
};
use winit::window::CursorIcon;
use winit::{
    dpi::PhysicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowBuilder},
};

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-1.0, -1.0, 0.0],
        tex_coords: [0.0, 1.0],
    },
    Vertex {
        position: [-1.0, 1.0, 0.0],
        tex_coords: [0.0, 0.0],
    },
    Vertex {
        position: [1.0, -1.0, 0.0],
        tex_coords: [1.0, 1.0],
    },
    Vertex {
        position: [1.0, 1.0, 0.0],
        tex_coords: [1.0, 0.0],
    },
];

const INDICES: &[u16] = &[
    0, 1, 2, // first triangle
    2, 1, 3, // second triangle
];

struct State<'a> {
    window: &'a Window,
    gpu_device: GPUDevice<'a>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    num_indices: u32,
    image_shader: Shader,

    mouse_state: MouseState,
    shape_stack: ShapeStack,
}

impl<'a> State<'a> {
    async fn new(window: &'a Window, png: &'a Png) -> Result<State<'a>> {
        let gpu_device = GPUDevice::new(window).await?;

        let texture_resource = gpu_device.create_texture("png_image", png)?;

        let feature_uniform_resource = {
            let (width, height) = gpu_device.surface_dimension();
            let feature_uniform = FeatureUniform::new(width, height, png.gamma);
            gpu_device.create_uniform::<FeatureUniform>(
                "feature_uniform",
                UniformBufferType::Feature(feature_uniform),
            )?
        };

        let draw_uniform_resource = {
            let draw_uniform = DrawUniform::new();
            gpu_device.create_uniform::<DrawUniform>(
                "draw_uniform",
                UniformBufferType::Draw(draw_uniform),
            )?
        };

        let image_shader = gpu_device.create_shader(
            "image",
            include_str!("image_shader.wgsl"),
            [
                texture_resource,
                feature_uniform_resource,
                draw_uniform_resource,
            ],
        )?;

        let vertex_buffer = gpu_device.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = gpu_device.device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });
        let num_indices = INDICES.len() as u32;

        let mouse_state = MouseState::default();
        let shape_stack = ShapeStack::new();

        Ok(Self {
            window,
            gpu_device,
            vertex_buffer,
            index_buffer,
            num_indices,
            image_shader,
            mouse_state,
            shape_stack,
        })
    }

    pub const fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }

        self.gpu_device.resize(new_size);

        let ShaderResourceType::Uniform(_, UniformBufferType::Feature(mut feature_uniform)) =
            self.image_shader.resources[1].resource
        else {
            panic!("Can not find feature uniform");
        };

        feature_uniform.update_window_dimensions(new_size.width, new_size.height)
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        let ShaderResourceType::Uniform(_, UniformBufferType::Feature(feature_uniform)) =
            &mut self.image_shader.resources[1].resource
        else {
            panic!("")
        };

        let ShaderResourceType::Uniform(_, UniformBufferType::Draw(draw_uniform)) =
            &mut self.image_shader.resources[2].resource
        else {
            panic!()
        };

        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    let prev_state = self.mouse_state.pressed();

                    self.mouse_state
                        .set_pressed(matches!(state, ElementState::Pressed));

                    if !draw_uniform.crosshair() {
                        return true;
                    }

                    match (prev_state, self.mouse_state.pressed()) {
                        (false, true) => {
                            let (start_x, start_y) = self.mouse_state.position();

                            dbg!("start drag", start_x, start_y);
                            self.mouse_state.set_start_drag(Some((start_x, start_y)));
                            draw_uniform.set_circle_center(start_x, start_y);
                        }
                        (true, false) => {
                            let initial_drag_position = self.mouse_state.start_drag();

                            if initial_drag_position.is_none() {
                                panic!("Logic error occured. Mouse state once finished pressing doesn't have initial drag position set.");
                            }

                            let (x, y) = initial_drag_position.unwrap();
                            let (edge_x, edge_y) = self.mouse_state.position();
                            let radius = compute_radius((x, y), (edge_x, edge_y));
                            self.shape_stack.push(Shape::Circle { x, y, radius });

                            // clear state
                            self.mouse_state.set_start_drag(None);
                            dbg!("stop drag");
                            draw_uniform.set_circle_radius(0.0);
                        }
                        _ => {}
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);

                if let Some(center) = self.mouse_state.start_drag() {
                    let radius = compute_radius(center, (x, y));
                    dbg!("dragging: radius", radius);
                    draw_uniform.set_circle_radius(radius);
                }

                self.mouse_state.update_position(x, y);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        physical_key: PhysicalKey::Code(keycode),
                        ..
                    },
                ..
            } => match (keycode, state) {
                (KeyCode::KeyA, ElementState::Pressed) => {
                    draw_uniform.toggle_crosshair();

                    if draw_uniform.crosshair() {
                        self.window.set_cursor_icon(CursorIcon::Crosshair);
                    } else {
                        self.window.set_cursor_icon(CursorIcon::Default);
                    }
                }
                (KeyCode::KeyC, ElementState::Pressed) => {
                    feature_uniform.reset_features();
                }
                (KeyCode::KeyB, ElementState::Pressed) => {
                    feature_uniform.toggle_blur();
                }
                (KeyCode::ArrowUp, ElementState::Pressed) => {
                    if feature_uniform.blur() {
                        feature_uniform.increase_blur_radius();
                    }

                    if feature_uniform.sharpen() {
                        feature_uniform.increase_sharpen_factor();
                    }
                }
                (KeyCode::ArrowDown, ElementState::Pressed) => {
                    if feature_uniform.blur() {
                        feature_uniform.decrease_blur_radius();
                    }

                    if feature_uniform.sharpen() {
                        feature_uniform.decrease_sharpen_factor();
                    }
                }
                (KeyCode::KeyG, ElementState::Pressed) => {
                    feature_uniform.toggle_grayscale();
                }
                (KeyCode::KeyS, ElementState::Pressed) => {
                    feature_uniform.toggle_sharpen();
                }
                (KeyCode::KeyI, ElementState::Pressed) => {
                    feature_uniform.toggle_invert();
                }
                (KeyCode::KeyE, ElementState::Pressed) => {
                    feature_uniform.toggle_edge_detect();
                }
                (KeyCode::KeyX, ElementState::Pressed) => {
                    feature_uniform.apply_transform(TransformAction::FlipX);
                }
                (KeyCode::KeyY, ElementState::Pressed) => {
                    feature_uniform.apply_transform(TransformAction::FlipY);
                }
                _ => return false,
            },
            _ => return false,
        }

        dbg!(&self.image_shader.resources);

        true
    }

    fn update(&self) {
        self.gpu_device.update_uniform(&self.image_shader);
    }

    fn render(&self) -> Result<(), SurfaceError> {
        let output = self.gpu_device.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&TextureViewDescriptor::default());

        let mut encoder =
            self.gpu_device
                .device
                .create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.image_shader.render_pipeline);

            let _ = &self
                .image_shader
                .resources
                .iter()
                .enumerate()
                .for_each(|(i, resource)| {
                    let bind_group = &resource.bind_group;
                    render_pass.set_bind_group(i as u32, &bind_group, &[]);
                });

            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);

            // self.shape_stack.shapes().iter().for_each(|shape| {
            //     let &Shape::Circle { x, y, radius } = shape;
            //
            //     let shape_uniform = DrawUniform {
            //         crosshair: self.draw_uniform.crosshair,
            //         circle_center_x: x,
            //         circle_center_y: y,
            //         circle_radius: radius,
            //     };
            //
            //     self.queue.write_buffer(
            //         &self.draw_buffer,
            //         0,
            //         bytemuck::cast_slice(&[shape_uniform]),
            //     );
            //
            //     render_pass.set_bind_group(2, &self.draw_bind_group, &[]);
            // });

            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.gpu_device.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

#[allow(clippy::future_not_send)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run(png: Png) -> anyhow::Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new()?;

    let (width, height) = png.dimensions();

    let window = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(width, height))
        .with_title("iris")
        .build(&event_loop)?;

    #[cfg(target_arch = "wasm32")]
    {
        // Winit prevents sizing with CSS, so we have to set
        // the size manually when on web.
        use winit::dpi::PhysicalSize;
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));

        use winit::platform::web::WindowExtWebSys;
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| {
                let dst = doc.get_element_by_id("wasm-example")?;
                let canvas = web_sys::Element::from(window.canvas()?);
                dst.append_child(&canvas).ok()?;
                Some(())
            })
            .expect("Couldn't append canvas to document body.");
    }

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window, &png).await?;
    let mut surface_configured = false;

    event_loop.run(move |event, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            event:
                                KeyEvent {
                                    state: ElementState::Pressed,
                                    physical_key: PhysicalKey::Code(KeyCode::Escape),
                                    ..
                                },
                            ..
                        } => control_flow.exit(),
                        WindowEvent::Resized(physical_size) => {
                            surface_configured = true;
                            state.resize(*physical_size);
                        }
                        WindowEvent::RedrawRequested => {
                            // This tells winit that we want another frame after this one
                            state.window().request_redraw();

                            if !surface_configured {
                                return;
                            }

                            state.update();
                            match state.render() {
                                Ok(_) => {}
                                // Reconfigure the surface if it's lost or outdated
                                Err(SurfaceError::Lost | SurfaceError::Outdated) => {
                                    state.resize(state.gpu_device.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(SurfaceError::OutOfMemory) => {
                                    log::error!("OutOfMemory");
                                    control_flow.exit();
                                }

                                // This happens when a frame takes too long to present
                                Err(SurfaceError::Timeout) => {
                                    log::warn!("Surface timeout")
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    })?;

    Ok(())
}
