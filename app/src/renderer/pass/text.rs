 
pub mod font;
pub mod glyphon_state; 
pub mod measurer;

use base::ui::text::{
    font::TextFont, 
    item::TextItem, 
    params::{TextParam, TextParams}, 
    style::{TextAlign, TextStyle}
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

    fn approx_char_width(style: &TextStyle) -> f32 {
        // tuned for monospace fonts; adjust if needed
        style.font_size * 0.62
    }

    fn compose_group_text(param: &TextParam) -> (String, f32, f32) {
        if param.items.is_empty() {
            return (String::new(), 0.0, 0.0);
        }

        let mut items: Vec<&TextItem> = param.items.iter().collect();

        let left = items
            .iter()
            .map(|i| i.x)
            .fold(f32::INFINITY, |a, b| a.min(b));
        let top = items
            .iter()
            .map(|i| i.y)
            .fold(f32::INFINITY, |a, b| a.min(b));

        items.sort_by(|a, b| {
            a.y.partial_cmp(&b.y)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
        });

        let line_h = param.style.line_height.max(1.0);
        let char_w = Self::approx_char_width(&param.style).max(1.0);

        let mut lines: Vec<String> = Vec::new();

        for item in items {
            let row = ((item.y - top) / line_h).round().max(0.0) as usize;
            let col = ((item.x - left) / char_w).round().max(0.0) as usize;

            if lines.len() <= row {
                lines.resize_with(row + 1, String::new);
            }

            let line = &mut lines[row];
            let current_cols = line.chars().count();
            if current_cols < col {
                line.push_str(&" ".repeat(col - current_cols));
            }
            line.push_str(&item.text);
        }

        (lines.join("\n"), left, top)
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

    fn rebuild_groups(
        &mut self,
        font_system: &mut glyphon::FontSystem,
        width: u32,
        height: u32,
    ) -> Vec<GlyphonGroup> {
        let mut out = Vec::with_capacity(self.params.groups.len());

        for group in &self.params.groups {
            if group.items.is_empty() {
                continue;
            }

            let attrs = group.style.font.attrs();
            let (text, left, top) = Self::compose_group_text(group);

            let mut buffer = glyphon::Buffer::new(
                font_system,
                glyphon::Metrics::new(group.style.font_size, group.style.line_height),
            );

            buffer.set_size(font_system, Some(width as f32), Some(height as f32));
            buffer.set_text(
                font_system,
                &text,
                &attrs,
                glyphon::Shaping::Advanced,
                Some(Self::to_cosmic_align(group.style.align)),
            );

            out.push(GlyphonGroup {
                buffer,
                left,
                top,
                color: group.style.color.to_array(),
            });
        }

        out
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

            for group in &mut state.groups {
                group.buffer.set_size(
                    &mut state.font_system,
                    Some(config.width as f32),
                    Some(config.height as f32),
                );
            }

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

        for variant in TextFont::all() {
            font_system
                .db_mut()
                .load_font_data(variant.font_bytes().to_vec());
        }

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

        let groups = self.rebuild_groups(&mut font_system, config.width, config.height);

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
                width: config.width,
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