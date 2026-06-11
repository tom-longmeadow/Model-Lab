use crate::{
    graphics_context::renderer::Renderer,
    ui::font::TextFontExt, // Your extension trait for converting to glyphon::Attrs
};
use base::ui::text::params::TextParams; // The REAL TextParams
use glyphon::{
    Buffer, Cache, Color, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer as GlyphonRenderer, Viewport,
};
use crate::ui::font::to_glyphon_align;

/// Internal state for the TextRenderer, encapsulating all `glyphon` components.
struct GlyphonState {
    font_system: FontSystem,
    swash_cache: SwashCache,
    _cache: Cache,
    atlas: TextAtlas,
    viewport: Viewport,
    renderer: GlyphonRenderer,
    // A text buffer that we will reuse every frame.
    buffer: Buffer,
}

/// A dedicated renderer that knows how to draw `TextParams` using `glyphon`.
pub struct TextRenderer {
    state: Option<GlyphonState>,
}

impl TextRenderer {
    pub fn new() -> Self {
        Self { state: None }
    }
}

impl Renderer<TextParams> for TextRenderer {
    /// Prepares the long-lived `glyphon` resources.
     fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        if self.state.is_some() {
            return;
        }

               let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let mut cache = Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &mut cache, config.format);
        // Remove the format argument, as it's no longer expected.
        let viewport = Viewport::new(device, &mut cache);
        let renderer =
            GlyphonRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);
        let buffer = Buffer::new(&mut font_system, Metrics::new(30.0, 42.0));

        self.state = Some(GlyphonState {
            font_system,
            swash_cache,
            _cache: cache,  
            atlas,
            viewport,
            renderer,
            buffer,
        });
    }

    /// Updates the text buffer and prepares glyphs for rendering.
   fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
        data: &TextParams,
    ) {
        let Some(state) = &mut self.state else {
            return;
        };

        state.viewport.update(
            queue,
            Resolution {
                width: config.width,
                height: config.height,
            },
        );

        // 1. MUTATING PHASE: Populate the single buffer with all text content.
        state.buffer.set_size(
            &mut state.font_system,
            Some(config.width as f32),
            Some(config.height as f32),
        );

        for group in &data.groups {
            let color = group.style.color;
            for item in &group.items {
                // Move Attrs creation inside the inner loop.
                // This is very cheap and solves the move error.
                let attrs = group.style.font.attrs();
                state.buffer.set_text(
                    &mut state.font_system,
                    &item.text,
                    &attrs.color(Color::rgba(color.r, color.g, color.b, color.a)),
                    Shaping::Advanced,
                    Some(to_glyphon_align(group.style.align)),
                );
            }
        }

        // 2. IMMUTABLE BORROWING PHASE: (This part is unchanged and correct)
        let mut areas: Vec<TextArea> = Vec::new();
        for group in &data.groups {
            let color = group.style.color;
            for item in &group.items {
                areas.push(TextArea {
                    buffer: &state.buffer,
                    left: item.x,
                    top: item.y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: item.x as i32,
                        top: item.y as i32,
                        right: (item.x + item.width) as i32,
                        bottom: config.height as i32,
                    },
                    default_color: Color::rgba(color.r, color.g, color.b, color.a),
                    custom_glyphs: &[],
                });
            }
        }

        // 3. PREPARE FOR RENDERING (Unchanged)
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
            .unwrap();
    }

    /// Issues the draw calls to render the prepared text.
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        let Some(state) = &self.state else {
            return;
        };

        // The actual draw call is delegated to glyphon's renderer.
        state
            .renderer
            .render(&state.atlas, &state.viewport, pass)
            .ok();
    }
}


// use crate::{graphics_context::renderer::Renderer, ui::text::GlyphonState};
// use crate::ui::text::TextParams;
// use glyphon::{
//     Attrs, Color, Family, FontSystem, Metrics, Shaping, SwashCache, TextAtlas,
//      Viewport,
// };

// /// A dedicated renderer that knows how to draw `TextParams` using `glyphon`.
// /// This struct is a wrapper around the complex `GlyphonState`.
// pub struct TextRenderer {
//     state: Option<GlyphonState>,
// }

// impl TextRenderer {
//     pub fn new() -> Self {
//         Self { state: None }
//     }
// }

// impl Renderer<TextParams> for TextRenderer {
//     /// Prepares the long-lived `glyphon` resources like the font system, caches,
//     /// and texture atlas. This is called once when the pass is created.
//     fn prepare(&mut self, device: &wgpu::Device, config: &wgpu::SurfaceConfiguration) {
//         if self.state.is_some() {
//             return;
//         }

//         let font_system = FontSystem::new();
//         let swash_cache = SwashCache::new();
//         let viewport = Viewport::new(device, config.format);
//         let atlas = TextAtlas::new(
//             device,
//             &viewport.queue,
//             viewport.format,
//             glyphon::AtlasSize::default(),
//         );
//         let renderer =
//             glyphon::TextRenderer::new(atlas, device, wgpu::MultisampleState::default(), None);

//         self.state = Some(GlyphonState {
//             font_system,
//             swash_cache,
//             viewport,
//             atlas: renderer.atlas, // The renderer takes ownership of the atlas
//             renderer,
//             buffer: glyphon::Buffer::new(
//                 &mut FontSystem::new(),
//                 Metrics::new(30.0, 42.0),
//             ),
//         });
//     }

//     /// Updates the text buffer with the latest data for the current frame and
//     /// tells `glyphon` to prepare the glyphs for rendering.
//     fn update(&mut self, device: &wgpu::Device, queue: &wgpu::Queue, config: &wgpu::SurfaceConfiguration, data: &TextParams) {
//         let Some(state) = &mut self.state else {
//             return;
//         };

//         // Update viewport dimensions from the config, not the data struct
//         state.viewport.update(
//             queue,
//             wgpu::Extent3d {
//                 width: config.width,
//                 height: config.height,
//                 depth_or_array_layers: 1,
//             },
//         );

//         // Clear the buffer from the previous frame
//         state.buffer.set_size(
//             &mut state.font_system,
//             data.screen_width as f32,
//             data.screen_height as f32,
//         );
//         state.buffer.clear();

//         // Add the new text areas for this frame
//         for group in &data.groups {
//             state.buffer.set_text(
//                 &mut state.font_system,
//                 group.text,
//                 Attrs::new().family(Family::SansSerif).color(Color::rgb(
//                     group.color[0],
//                     group.color[1],
//                     group.color[2],
//                 )),
//                 Shaping::Advanced,
//             );
//         }

//         // Prepare the text for rendering. This performs shaping, layout, and updates
//         // the texture atlas, uploading new glyphs to the GPU.
//         state
//             .renderer
//             .prepare(
//                 device,
//                 queue,
//                 &mut state.font_system,
//                 &mut state.atlas,
//                 &state.viewport,
//                 [state.buffer.clone()], // Glyphon's prepare takes a slice of buffers
//                 &mut state.swash_cache,
//             )
//             .unwrap();
//     }

//     /// Issues the draw calls to render the prepared text.
//     fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
//         let Some(state) = &self.state else {
//             return;
//         };

//         // The actual draw call is delegated to glyphon's renderer.
//         state
//             .renderer
//             .render(&state.atlas, &state.viewport, pass)
//             .ok();
//     }
// }