use crate::png::grammar::Png;
use crate::renderer::draw_uniform::DrawUniform;
use crate::renderer::feature_uniform::{FeatureUniform, TransformAction};
use crate::renderer::gpu_state::{GpuResourceAllocator, Shader};
use crate::renderer::mouse_state::MouseState;
use crate::renderer::shape::{compute_radius, Shape, ShapeStack};
use anyhow::Result;
use wgpu::SurfaceError;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::EventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{CursorIcon, Window, WindowBuilder};

/// AppState is the state that is created by user input.
#[derive(Debug)]
pub struct AppState<'a> {
    pub gpu_allocator: GpuResourceAllocator<'a>,

    // pub png: &'a Png,
    pub window: &'a Window,
    pub(crate) size: PhysicalSize<u32>,

    pub feature_uniform: FeatureUniform,
    pub draw_uniform: DrawUniform,
    pub mouse_state: MouseState,
    pub shape_stack: ShapeStack,

    pub image_shader: Shader,
}

impl<'a> AppState<'a> {
    pub async fn new(window: &'a Window, png: &'a Png) -> Result<AppState<'a>> {
        let gpu_allocator = GpuResourceAllocator::new(window).await?;

        let size = window.inner_size();

        let image_texture_resource = gpu_allocator.create_texture_resource("image_texture", png)?;

        let feature_uniform = { FeatureUniform::new(size.width, size.height, png.gamma) };
        let feature_uniform_resource =
            gpu_allocator.create_uniform_resource("feature_uniform", feature_uniform)?;

        let draw_uniform = DrawUniform::new();
        let draw_uniform_resource =
            gpu_allocator.create_uniform_resource("draw_uniform", draw_uniform)?;

        let image_shader = gpu_allocator.create_shader(
            "image_shader",
            include_str!("image_shader.wgsl"),
            vec![image_texture_resource],
            vec![feature_uniform_resource, draw_uniform_resource],
        );

        let mouse_state = MouseState::default();
        let shape_stack = ShapeStack::new();

        Ok(Self {
            gpu_allocator,
            // png,
            window,
            size,
            feature_uniform,
            draw_uniform,
            mouse_state,
            shape_stack,
            image_shader,
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
                    self.draw_uniform.set_circle_radius(radius);
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

        true
    }

    pub(crate) fn update(&self) {
        // Make sure the uniform resource at index i correctly matches up to the uniform structure.
        let uniform_resources = &self.image_shader.uniform_resources;

        self.gpu_allocator
            .write_uniform_buffer(&uniform_resources[0].resource, self.feature_uniform);

        self.gpu_allocator
            .write_uniform_buffer(&uniform_resources[1].resource, self.draw_uniform);
    }

    pub(crate) fn render(&self) -> Result<(), SurfaceError> {
        self.gpu_allocator.render(&self.image_shader)?;

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
    let mut state = AppState::new(&window, &png).await?;
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
                                    state.resize(state.size)
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
