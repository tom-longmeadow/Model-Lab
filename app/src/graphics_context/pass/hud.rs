use std::sync::{Arc, Mutex};
use std::time::Instant;

use base::ui::{
    layout::{color::Color, edge_insets::EdgeInsets, rect::Rect},
    text::{
        font::TextFont,
        item::TextItem,
        params::{TextGroup, TextParams},
        style::{TextAlign, TextStyle},
    },
};
use crate::graphics_context::{
    pass::Pass,
    renderer::{text::TextRenderer, Renderer},
};

/// A named metric row displayed in the HUD.
/// The producer is responsible for formatting the value string including units.
#[derive(Clone)]
pub struct HudMetric {
    pub label: String,
    pub value: String,
}

/// Shared metrics state written by any system and read by [`HudPass`].
///
/// Metrics are stored in insertion order and rendered top-to-bottom.
/// Call [`HudState::set`] to insert or update a named metric.
pub struct HudState {
    metrics: Vec<HudMetric>,
}

impl Default for HudState {
    fn default() -> Self {
        Self { metrics: Vec::new() }
    }
}

impl HudState {
    /// Insert or update a metric by label. Preserves insertion order for new labels.
    pub fn set(&mut self, label: impl Into<String>, value: impl Into<String>) {
        let label = label.into();
        let value = value.into();
        if let Some(m) = self.metrics.iter_mut().find(|m| m.label == label) {
            m.value = value;
        } else {
            self.metrics.push(HudMetric { label, value });
        }
    }

    /// Remove a metric by label. No-op if not present.
    pub fn remove(&mut self, label: &str) {
        self.metrics.retain(|m| m.label != label);
    }

    pub fn metrics(&self) -> &[HudMetric] {
        &self.metrics
    }
}

/// A render pass that displays a live HUD overlay driven entirely by [`HudState`].
///
/// `HudPass` measures its own frame time and writes "FPS" and "Frame" into the
/// shared state automatically. All other metrics are written externally:
///
pub struct HudPass {
    state: Arc<Mutex<HudState>>,
    renderer: TextRenderer,
    last_frame: Instant,
}

impl HudPass {
    pub fn new(state: Arc<Mutex<HudState>>) -> Self {
        Self {
            state,
            renderer: TextRenderer::new(TextParams { groups: vec![] }),
            last_frame: Instant::now(),
        }
    }

    fn hud_style() -> TextStyle {
        TextStyle {
            font_size: 18.0,
            line_height: 24.0,
            font: TextFont::Regular,
            color: Color { r: 255, g: 255, b: 255, a: 255 },
            align: TextAlign::Left,
        }
    }

    fn build_params(metrics: &[HudMetric], config: &wgpu::SurfaceConfiguration) -> TextParams {
        let style   = Self::hud_style();
        let padding = EdgeInsets::none();
        let label_w = 100.0_f32;
        let value_w = 160.0_f32;
        let value_x = 112.0_f32;
        let row_h   = 24.0_f32;
        let bottom  = config.height as f32 - 12.0;
        let n       = metrics.len();

        let groups = metrics.iter().enumerate().map(|(i, m)| {
            let y = bottom - (n - i) as f32 * row_h;
            TextGroup {
                style,
                items: vec![
                    TextItem::new(&m.label, Rect { x: 8.0,    y, w: label_w, h: row_h }, padding),
                    TextItem::new(&m.value, Rect { x: value_x, y, w: value_w, h: row_h }, padding),
                ],
            }
        }).collect();

        TextParams { groups }
    }
}

impl Pass for HudPass {
    fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        self.renderer.prepare(device, queue, config);
    }

    fn update(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        config: &wgpu::SurfaceConfiguration,
    ) {
        let now = Instant::now();
        let dt  = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;

        if let Ok(mut s) = self.state.try_lock() {
            s.set("FPS",   format!("{:.0}",    if dt > 0.0 { 1.0 / dt } else { 0.0 }));
            s.set("Frame", format!("{:.2} ms", dt * 1000.0));
        }

        let params = self.state.lock()
            .map(|s| Self::build_params(s.metrics(), config))
            .unwrap_or(TextParams { groups: vec![] });

        self.renderer.update_data(params);
        self.renderer.update(device, queue, config);
    }

    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        self.renderer.draw(pass);
    }
}
