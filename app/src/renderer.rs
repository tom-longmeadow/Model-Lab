
use std::sync::Arc;

use wgpu::rwh::{HasWindowHandle, HasDisplayHandle};

pub mod config;
pub mod error;
pub mod pass;

use crate::renderer::{
    config::RendererConfig,
    error::RendererError,
    pass::RenderPass,
};


pub struct Renderer {
    surface:      wgpu::Surface<'static>,
    device:       wgpu::Device,
    queue:        wgpu::Queue,
    surface_cfg: wgpu::SurfaceConfiguration,
    config:  RendererConfig,              
    passes:       Vec<Box<dyn RenderPass>>,
}


impl Renderer {

    pub async fn new(
        window: Arc<impl HasWindowHandle + HasDisplayHandle + Send + Sync + 'static>,
        width:  u32,
        height: u32,
        config: RendererConfig,
    ) -> Result<Self, RendererError> {
        tracing::info!("Renderer initializing {}x{}", width, height);

        let instance = wgpu::Instance::default();
        let surface  = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .map_err(|_| RendererError::NoAdapter)?;

        tracing::debug!("Adapter: {}", adapter.get_info().name);
        tracing::debug!("Backend: {:?}", adapter.get_info().backend);

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(RendererError::DeviceRequest)?;

        let caps       = surface.get_capabilities(&adapter);
        let format     = caps.formats.first().copied().ok_or(RendererError::NoSupportedFormat)?;
        let alpha_mode = caps.alpha_modes.first().copied()
            .unwrap_or(wgpu::CompositeAlphaMode::Auto);

        tracing::debug!("Surface format: {:?}", format);
        tracing::debug!("Present mode:   {:?}", config.present_mode);

        let surface_cfg = wgpu::SurfaceConfiguration {
            usage:        wgpu::TextureUsages::RENDER_ATTACHMENT,
            format,
            width,
            height,
            present_mode: config.present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_cfg);

        tracing::info!("Renderer ready");
        Ok(Self { surface, device, queue, surface_cfg, config, passes: vec![] })
    }

    pub fn add_pass(&mut self, mut pass: Box<dyn RenderPass>) {
        pass.prepare(&self.device, &self.queue, &self.surface_cfg);
        self.passes.push(pass);
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    pub fn surface_config(&self) -> &wgpu::SurfaceConfiguration {
        &self.surface_cfg
    }
    
    pub fn render(&mut self) -> Result<(), RendererError> {
        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(t) => t,
            wgpu::CurrentSurfaceTexture::Suboptimal(t) => t,
            wgpu::CurrentSurfaceTexture::Timeout => return Err(RendererError::Timeout),
            wgpu::CurrentSurfaceTexture::Occluded => return Err(RendererError::Occluded),
            wgpu::CurrentSurfaceTexture::Outdated => return Err(RendererError::Outdated),
            wgpu::CurrentSurfaceTexture::Lost => return Err(RendererError::Lost),
            wgpu::CurrentSurfaceTexture::Validation => return Err(RendererError::Validation),
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        for pass in &mut self.passes {
            pass.update(&self.device, &self.queue);
        }

        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("renderer.encoder"),
                });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("renderer.main_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.config.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            for pass in &mut self.passes {
                pass.draw(&mut render_pass);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
        Ok(())
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            tracing::debug!("Resize ignored — zero dimension {}x{}", width, height);
            return;
        }
        tracing::debug!("Renderer resizing to {}x{}", width, height);
        self.surface_cfg.width  = width;
        self.surface_cfg.height = height;
        self.reconfigure();
        for pass in &mut self.passes {
            pass.prepare(&self.device, &self.queue, &self.surface_cfg);
        }
    }

    pub fn reconfigure(&mut self) {
        tracing::debug!("Surface reconfiguring");
        self.surface.configure(&self.device, &self.surface_cfg);
    }



}