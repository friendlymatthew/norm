use crate::{
    image::grammar::Image,
    renderer::{
        draw_uniform::DrawUniform,
        feature_uniform::{FeatureUniform, TransformAction},
        gpu_state::GpuResourceAllocator,
        mouse_state::MouseState,
        shader::{Shader, TextureResource},
        shape::{compute_distance, Circle, EditorState, Shape},
        shape_uniform::{CircleData, ShapeUniform, MAX_CIRCLES},
    },
};
use anyhow::Result;
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyEvent, Modifiers, MouseButton, WindowEvent},
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorIcon, Window, WindowBuilder},
};

/// AppState is the state that is created by user input.
#[derive(Debug)]
pub struct AppState<'a> {
    pub gpu_allocator: GpuResourceAllocator<'a>,

    pub window: &'a Window,
    pub(crate) size: PhysicalSize<u32>,

    pub feature_uniform: FeatureUniform,
    pub draw_uniform: DrawUniform,
    pub mouse_state: MouseState,
    pub editor_state: EditorState,
    pub modifiers: Modifiers,

    pub image_shader: Shader,
    pub shape_shader: Shader,
    pub shape_render_texture: TextureResource,
    pub shape_uniform: ShapeUniform,
    pub circle_storage_buffer: wgpu::Buffer,
}

impl<'a> AppState<'a> {
    pub async fn new(window: &'a Window, image: &'a Image) -> Result<AppState<'a>> {
        let gpu_allocator = GpuResourceAllocator::new(window).await?;

        let size = window.inner_size();

        let image_texture_resource =
            gpu_allocator.create_texture_resource("image_texture", image)?;

        let feature_uniform = { FeatureUniform::new(size.width, size.height, image.gamma()) };
        let feature_uniform_resource =
            gpu_allocator.create_uniform_resource("feature_uniform", feature_uniform)?;

        let draw_uniform = DrawUniform::new();
        let draw_uniform_resource =
            gpu_allocator.create_uniform_resource("draw_uniform", draw_uniform)?;

        // Create shape render texture
        let shape_render_texture =
            gpu_allocator.create_render_texture("shape_texture", size.width, size.height);

        // Create shape uniform and storage buffer
        let shape_uniform = ShapeUniform::new(size.width, size.height);
        let shape_uniform_resource =
            gpu_allocator.create_uniform_resource("shape_uniform", shape_uniform)?;

        // Create empty circle storage buffer
        let empty_circles = vec![CircleData::default(); MAX_CIRCLES];
        let circle_storage_buffer =
            gpu_allocator.create_storage_buffer("circle_storage", &empty_circles)?;

        let shape_shader = gpu_allocator.create_shape_shader(
            "shape_shader",
            include_str!("shape_shader.wgsl"),
            shape_uniform_resource,
            &circle_storage_buffer,
        );

        // Create a reference to the shape texture for the image shader
        let shape_texture_for_image = gpu_allocator.create_texture_resource_from_existing(
            "shape_texture_ref",
            &shape_render_texture.resource,
        );

        let image_shader = gpu_allocator.create_shader(
            "image_shader",
            include_str!("image_shader.wgsl"),
            vec![image_texture_resource, shape_texture_for_image],
            vec![feature_uniform_resource, draw_uniform_resource],
        );

        let mouse_state = MouseState::default();
        let editor_state = EditorState::default();
        let modifiers = Modifiers::default();

        Ok(Self {
            gpu_allocator,
            window,
            size,
            feature_uniform,
            draw_uniform,
            mouse_state,
            editor_state,
            modifiers,
            image_shader,
            shape_shader,
            shape_render_texture,
            shape_uniform,
            circle_storage_buffer,
        })
    }

    pub const fn window(&self) -> &Window {
        self.window
    }

    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.gpu_allocator.configure_surface(&new_size);
            self.feature_uniform
                .update_window_dimensions(new_size.width, new_size.height);
            self.shape_uniform
                .update_dimensions(new_size.width, new_size.height);
        }
    }

    pub(crate) fn input(&mut self, event: &WindowEvent) -> bool {
        let feature_uniform = &mut self.feature_uniform;
        let draw_uniform = &mut self.draw_uniform;

        match event {
            WindowEvent::MouseInput { state, button, .. } => {
                if *button == MouseButton::Left {
                    let prev_state = self.mouse_state.pressed();
                    self.mouse_state
                        .set_pressed(matches!(state, ElementState::Pressed));

                    if draw_uniform.crosshair() {
                        // Crosshair mode: Draw new circles
                        match (prev_state, self.mouse_state.pressed()) {
                            (false, true) => {
                                let (start_x, start_y) = self.mouse_state.position();
                                self.mouse_state.set_start_drag(Some((start_x, start_y)));
                                draw_uniform.set_circle_center(start_x, start_y);
                            }
                            (true, false) => {
                                let initial_drag_position = self.mouse_state.start_drag();
                                if initial_drag_position.is_none() {
                                    panic!("Logic error occurred. Mouse state once finished pressing doesn't have initial drag position set.");
                                }

                                let pixel_coord = initial_drag_position.unwrap();
                                let (edge_x, edge_y) = self.mouse_state.position();
                                let radius = compute_distance(pixel_coord, (edge_x, edge_y));

                                let circle = Circle::from_pixel_coordinate(
                                    pixel_coord,
                                    radius,
                                    (self.size.width as f32, self.size.height as f32),
                                );

                                self.editor_state.push_shape(Shape::Circle(circle));

                                // Clear state
                                self.mouse_state.set_start_drag(None);
                                draw_uniform.set_circle_radius(0.0);
                            }
                            _ => {}
                        }
                    } else {
                        // Selection mode: Select and move circles
                        match (prev_state, self.mouse_state.pressed()) {
                            (false, true) => {
                                // Mouse press - check if we clicked on a circle
                                let mouse_coordinate = self.mouse_state.position();

                                if let Some(circle_index) =
                                    self.editor_state.shape_stack.find_shape_at_point(
                                        mouse_coordinate,
                                        self.size.width,
                                        self.size.height,
                                    )
                                {
                                    // Found a circle - select it and start dragging
                                    self.mouse_state.set_selected_shape(Some(circle_index));
                                    self.mouse_state.set_dragging_shape(true);

                                    // Calculate offset from circle center to mouse position
                                    let Shape::Circle(circle) =
                                        self.editor_state.shape_stack.get_unchecked(circle_index);

                                    let (x, y) = circle.center();

                                    let normalized_mouse_coord = {
                                        let (x_in_pixel_coords, y_in_pixel_coords) =
                                            mouse_coordinate;

                                        (
                                            x_in_pixel_coords / self.size.width as f32,
                                            y_in_pixel_coords / self.size.height as f32,
                                        )
                                    };

                                    let offset_x = normalized_mouse_coord.0 - x;
                                    let offset_y = normalized_mouse_coord.1 - y;

                                    self.mouse_state.set_drag_offset((offset_x, offset_y));
                                } else {
                                    // Clicked on empty space - deselect any selected circle
                                    self.mouse_state.set_selected_shape(None);
                                }
                            }
                            (true, false) => {
                                // Mouse release - stop dragging
                                self.mouse_state.set_dragging_shape(false);
                            }
                            _ => {}
                        }
                    }
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as f32, position.y as f32);

                if draw_uniform.crosshair() {
                    // Crosshair mode: Update the preview circle radius
                    if let Some(center) = self.mouse_state.start_drag() {
                        let radius = compute_distance(center, (x, y));
                        self.draw_uniform.set_circle_radius(radius);
                    }
                } else {
                    // Selection mode: Handle circle dragging
                    if self.mouse_state.dragging_shape() {
                        if let Some(selected_index) = self.mouse_state.selected_shape() {
                            let normalized_x = x / self.size.width as f32;
                            let normalized_y = y / self.size.height as f32;

                            // Apply the drag offset to maintain relative position
                            let (offset_x, offset_y) = self.mouse_state.drag_offset();
                            let new_x = normalized_x - offset_x;
                            let new_y = normalized_y - offset_y;

                            // Move the circle to the new position
                            self.editor_state
                                .shape_stack
                                .move_shape(selected_index, new_x, new_y);
                        }
                    }
                }

                self.mouse_state.update_position(x, y);
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                self.modifiers = *new_modifiers;
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
                (KeyCode::Delete, ElementState::Pressed)
                | (KeyCode::Backspace, ElementState::Pressed) => {
                    // Delete the selected circle
                    if let Some(selected_index) = self.mouse_state.selected_shape() {
                        self.editor_state.shape_stack.remove_shape(selected_index);
                        self.mouse_state.set_selected_shape(None);
                        self.mouse_state.set_dragging_shape(false);
                    }
                }
                (KeyCode::KeyZ, ElementState::Pressed) => {
                    #[cfg(target_os = "macos")]
                    if self.modifiers.state().super_key() {
                        self.editor_state.undo();
                    }

                    #[cfg(not(target_os = "macos"))]
                    if self.modifiers.state().control_key() {
                        self.revision_stack.undo();
                    }
                }
                _ => return false,
            },
            _ => return false,
        }

        true
    }

    pub(crate) fn update(&mut self) {
        // Update image shader uniforms
        let uniform_resources = &self.image_shader.uniform_resources;
        self.gpu_allocator
            .write_uniform_buffer(&uniform_resources[0].resource, self.feature_uniform);
        self.gpu_allocator
            .write_uniform_buffer(&uniform_resources[1].resource, self.draw_uniform);

        // Update shape data
        self.update_shape_data();
    }

    fn update_shape_data(&mut self) {
        let num_circles = self.editor_state.shape_stack.len().min(MAX_CIRCLES);

        self.shape_uniform.set_num_circles(num_circles as u32);
        self.shape_uniform
            .set_selected_circle(self.mouse_state.selected_shape());

        // Update shape uniform
        let shape_uniform_resources = &self.shape_shader.uniform_resources;
        self.gpu_allocator
            .write_uniform_buffer(&shape_uniform_resources[0].resource, self.shape_uniform);

        // Update circle storage buffer
        let mut circle_data = vec![CircleData::default(); MAX_CIRCLES];
        for (i, (_, shape)) in self
            .editor_state
            .shape_stack
            .shapes()
            .into_iter()
            .take(MAX_CIRCLES)
            .enumerate()
        {
            circle_data[i] = CircleData::from(shape);
        }

        self.gpu_allocator
            .write_storage_buffer(&self.circle_storage_buffer, &circle_data);
    }

    pub(crate) fn render(&self) -> Result<(), wgpu::SurfaceError> {
        let (output, view, mut encoder) = self.gpu_allocator.begin_frame()?;

        // First pass: Render shapes to shape texture
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shape render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.shape_render_texture.resource.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.shape_shader.render_pipeline);

            // Set bind group for shape shader (uniform + storage buffer)
            render_pass.set_bind_group(0, &self.shape_shader.uniform_resources[0].bind_group, &[]);

            render_pass.set_vertex_buffer(0, self.gpu_allocator.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.gpu_allocator.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..self.gpu_allocator.num_indices(), 0, 0..1);
        }

        // Second pass: Render final image with shapes composited
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("content render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            render_pass.set_pipeline(&self.image_shader.render_pipeline);

            let mut i = 0;

            self.image_shader.texture_resources.iter().for_each(|r| {
                render_pass.set_bind_group(i, &r.bind_group, &[]);
                i += 1;
            });

            self.image_shader.uniform_resources.iter().for_each(|r| {
                render_pass.set_bind_group(i, &r.bind_group, &[]);
                i += 1;
            });

            render_pass.set_vertex_buffer(0, self.gpu_allocator.vertex_buffer.slice(..));
            render_pass.set_index_buffer(
                self.gpu_allocator.index_buffer.slice(..),
                wgpu::IndexFormat::Uint16,
            );

            render_pass.draw_indexed(0..self.gpu_allocator.num_indices(), 0, 0..1);
        }

        self.gpu_allocator.end_frame(encoder);
        output.present();

        Ok(())
    }
}

#[allow(clippy::future_not_send)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
pub async fn run(image: Image) -> anyhow::Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");
        } else {
            env_logger::init();
        }
    }

    let event_loop = EventLoop::new()?;

    let (width, height) = image.dimensions();

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
    let mut state = AppState::new(&window, &image).await?;
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
                                Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                                    state.resize(state.size)
                                }
                                // The system is out of memory, we should probably quit
                                Err(wgpu::SurfaceError::OutOfMemory) => {
                                    log::error!("OutOfMemory");
                                    control_flow.exit();
                                }

                                // This happens when a frame takes too long to present
                                Err(wgpu::SurfaceError::Timeout) => {
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
