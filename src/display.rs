use glutin::{
    config::{ConfigTemplateBuilder, GlConfig},
    context::{ContextApi, ContextAttributesBuilder, PossiblyCurrentContext, Version},
    display::GetGlDisplay,
    prelude::*,
    surface::{Surface, SurfaceAttributesBuilder, WindowSurface},
};
use glutin_winit::DisplayBuilder;
use raw_window_handle::HasWindowHandle;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowAttributes},
};

pub struct GemDisplay {
    pub window: Window,
    pub gl_context: PossiblyCurrentContext,
    pub gl_surface: Surface<WindowSurface>,
}

impl GemDisplay {
    pub fn new(event_loop: &EventLoop<()>, width: u32, height: u32, title: &str) -> Self {
        let window_attrs = WindowAttributes::default()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(width, height))
            .with_resizable(true);

        let config_template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_depth_size(24);

        let (window, gl_config) = DisplayBuilder::new()
            .with_window_attributes(Some(window_attrs))
            .build(event_loop, config_template, |configs| {
                // Pick the config with the most samples
                configs
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .expect("No available configs")
            })
            .expect("Failed to create display and window");

        let window = window.expect("Failed to create window");

        println!(
            "[GemDisplay] Picked config with {} samples",
            gl_config.num_samples()
        );

        let gl_display = gl_config.display();

        // Create OpenGL context
        let context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::OpenGl(Some(Version::new(3, 3))))
            .build(Some(
                window
                    .window_handle()
                    .expect("Failed to get window handle")
                    .as_raw(),
            ));

        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(ContextApi::Gles(Some(Version::new(2, 0))))
            .build(Some(
                window
                    .window_handle()
                    .expect("Failed to get window handle")
                    .as_raw(),
            ));

        let not_current_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    println!("[GemDisplay] OpenGL 3.3 failed, falling back to GLES 2.0");
                    gl_display
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("Failed to create OpenGL context")
                })
        };

        // Create surface for the window
        let window_size = window.inner_size();
        let surface_attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window
                .window_handle()
                .expect("Failed to get window handle")
                .as_raw(),
            window_size.width.try_into().unwrap(),
            window_size.height.try_into().unwrap(),
        );

        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attrs)
                .expect("Failed to create window surface")
        };

        // Make context current
        let gl_context = not_current_context
            .make_current(&gl_surface)
            .expect("Failed to make context current");

        println!("[GemDisplay] OpenGL context created and made current");

        Self {
            window,
            gl_context,
            gl_surface,
        }
    }

    pub fn resize(&self, width: u32, height: u32) {
        self.gl_surface.resize(
            &self.gl_context,
            width.try_into().unwrap(),
            height.try_into().unwrap(),
        );
    }

    pub fn swap_buffers(&self) {
        self.gl_surface
            .swap_buffers(&self.gl_context)
            .expect("Failed to swap buffers");
    }
}
