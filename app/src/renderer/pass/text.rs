 
pub mod font;
pub mod glyphon_state; 
pub mod measurer;

use base::ui::text::{
    font::TextFont,  
    params::{TextParams}, 
    style::{TextAlign}
};
use glyphon::cosmic_text;
 
use crate::renderer::pass::{
    RenderPass, 
    text::{
        font::TextFontExt, 
        glyphon_state::{GlyphonState, GlyphonGroup}
    }
}; 
 
pub struct TextRenderPass {
    params: TextParams,
    state: Option<GlyphonState>,
}

impl TextRenderPass {
    pub fn new(params: TextParams) -> Self {
        Self { params, state: None }
    }

    fn rebuild_groups_with(
        params: &TextParams,
        font_system: &mut glyphon::FontSystem,
        width: u32,
        height: u32,
    ) -> Vec<GlyphonGroup> {
        let mut out = Vec::new();

        for param in &params.groups {
            let attrs = param.style.font.attrs();

            for item in &param.items {
                let buf_width = if item.width > 0.0 { item.width } else { width as f32 };

                let mut buffer = glyphon::Buffer::new(
                    font_system,
                    glyphon::Metrics::new(param.style.font_size, param.style.line_height),
                );

                buffer.set_size(font_system, Some(buf_width), Some(height as f32));
                buffer.set_text(
                    font_system,
                    &item.text,
                    &attrs,
                    glyphon::Shaping::Advanced,
                    Some(Self::to_cosmic_align(param.style.align)),
                );

                out.push(GlyphonGroup {
                    buffer,
                    left:      item.x,
                    top:       item.y,
                    buf_width,
                    color:     param.style.color.to_array(),
                });
            }
        }

        out
    }
 
    fn to_cosmic_align(align: TextAlign) -> cosmic_text::Align {
        match align {
            TextAlign::Left => cosmic_text::Align::Left,
            TextAlign::Center => cosmic_text::Align::Center,
            TextAlign::Right => cosmic_text::Align::Right,
            TextAlign::Justified => cosmic_text::Align::Justified,
            TextAlign::End => cosmic_text::Align::End,
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
        // rebuild groups on every prepare — params may have changed
        if let Some(state) = &mut self.state {
            state.width  = config.width;
            state.height = config.height;

            // rebuild with new params
            let groups = Self::rebuild_groups_with(
                &self.params,
                &mut state.font_system,
                config.width,
                config.height,
            );
            state.groups = groups;

            state.viewport.update(
                queue,
                glyphon::Resolution {
                    width:  config.width,
                    height: config.height,
                },
            );
            return;
        }

        // first time init
        let mut font_system = glyphon::FontSystem::new();

        for variant in TextFont::all() {
            font_system
                .db_mut()
                .load_font_data(variant.font_bytes().to_vec());
        }

        let swash_cache = glyphon::SwashCache::new();
        let cache       = glyphon::Cache::new(device);
        let mut atlas   = glyphon::TextAtlas::new(device, queue, &cache, config.format);
        let viewport    = glyphon::Viewport::new(device, &cache);
        let renderer    = glyphon::TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );

        let groups = Self::rebuild_groups_with(
            &self.params,
            &mut font_system,
            config.width,
            config.height,
        );

        let mut state = GlyphonState {
            width: config.width,
            height: config.height,
            font_system,
            swash_cache,
            viewport,
            atlas,
            renderer,
            groups,
        };

        state.viewport.update(
            queue,
            glyphon::Resolution {
                width:  config.width,
                height: config.height,
            },
        );

        self.state = Some(state);
    }

    fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue) {
        let Some(state) = &mut self.state else { return };

        state.viewport.update(
            queue,
            glyphon::Resolution {
                width: state.width,
                height: state.height,
            },
        );

        let mut areas = Vec::with_capacity(state.groups.len());
        for group in &state.groups {
            let [r, g, b, a] = group.color;

            areas.push(glyphon::TextArea {
                buffer: &group.buffer,
                left: group.left,
                top: group.top,
                scale: 1.0,
                bounds: glyphon::TextBounds {
                    left: 0,
                    top: 0,
                    right: state.width as i32,
                    bottom: state.height as i32,
                },
                default_color: cosmic_text::Color::rgba(r, g, b, a),
                custom_glyphs: &[],
            });
        }

        state
            .renderer
            .prepare(
                device,
                queue,
                &mut state.font_system,
                &mut state.atlas,
                &state.viewport,
                areas,
                &mut state.swash_cache,
            )
            .expect("glyphon prepare failed");
    }

    fn draw<'a>(&'a mut self, pass: &mut wgpu::RenderPass<'a>) {
        let Some(state) = &mut self.state else { return };
        state
            .renderer
            .render(&state.atlas, &state.viewport, pass)
            .expect("glyphon render failed");
    }
}