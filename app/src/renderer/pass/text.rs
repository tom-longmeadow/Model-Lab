use glyphon::cosmic_text;

use crate::renderer::pass::RenderPass;

pub struct TextRenderPass {
    text: String,
    state: Option<GlyphonState>,
}

struct GlyphonState {
    width: u32,
    height: u32,
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
    viewport: glyphon::Viewport,
    atlas: glyphon::TextAtlas,
    renderer: glyphon::TextRenderer,
    buffers: Vec<glyphon::Buffer>,
}

impl TextRenderPass {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            state: None,
        }
    }
}

impl RenderPass for TextRenderPass {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if let Some(state) = &mut self.state {
            state.width = config.width;
            state.height = config.height;

            state.buffers[0].set_size(
                &mut state.font_system,
                Some(config.width as f32),
                Some(config.height as f32),
            );

            state.viewport.update(
                queue,
                glyphon::Resolution {
                    width: config.width,
                    height: config.height,
                },
            );
            return;
        }

        let mut font_system = glyphon::FontSystem::new();
        let swash_cache = glyphon::SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let mut atlas = glyphon::TextAtlas::new(device, queue, &cache, config.format);
        let viewport = glyphon::Viewport::new(device, &cache);
        let renderer = glyphon::TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );

        let mut buffer = glyphon::Buffer::new(
            &mut font_system,
            glyphon::Metrics::new(24.0, 32.0),
        );

        buffer.set_size(
            &mut font_system,
            Some(config.width as f32),
            Some(config.height as f32),
        );

        let attrs = glyphon::Attrs::new();
        buffer.set_text(
            &mut font_system,
            &self.text,
            &attrs,
            glyphon::Shaping::Advanced,
            Some(cosmic_text::Align::Left),
        );

        let mut state = GlyphonState {
            width: config.width,
            height: config.height,
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            buffers: vec![buffer],
        };

        state.viewport.update(
            queue,
            glyphon::Resolution {
                width: config.width,
                height: config.height,
            },
        );

        self.state = Some(state);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let Some(state) = &mut self.state else {
            return;
        };

        let GlyphonState {
            width,
            height,
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            buffers,
        } = state;

        viewport.update(
            queue,
            glyphon::Resolution {
                width: *width,
                height: *height,
            },
        );

        let text_area = glyphon::TextArea {
            buffer: &buffers[0],
            left: 20.0,
            top: 20.0,
            scale: 1.0,
            bounds: glyphon::TextBounds {
                left: 0,
                top: 0,
                right: *width as i32,
                bottom: *height as i32,
            },
            default_color: cosmic_text::Color::rgb(255, 255, 255),
            custom_glyphs: &[],
        };

        renderer
            .prepare(
                device,
                queue,
                font_system,
                atlas,
                viewport,
                [text_area],
                swash_cache,
            )
            .expect("glyphon prepare failed");
    }

    fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>) {
        let Some(state) = &mut self.state else {
            return;
        };

        state
            .renderer
            .render(&state.atlas, &state.viewport, pass)
            .expect("glyphon render failed");
    }
}