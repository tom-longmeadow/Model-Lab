

pub enum RendererError {
    NoAdapter,
    NoSupportedFormat,
    DeviceRequest(wgpu::RequestDeviceError),
    Surface(wgpu::CreateSurfaceError),
    Timeout,
    Occluded,
    Outdated,
    Lost,
    Validation,
}

impl std::fmt::Display for RendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoAdapter         => write!(f, "no suitable GPU adapter found"),
            Self::NoSupportedFormat => write!(f, "no supported surface format found"),
            Self::DeviceRequest(e)  => write!(f, "failed to create device: {}", e),
            Self::Surface(e)        => write!(f, "failed to create surface: {}", e),
            Self::Timeout           => write!(f, "surface texture acquisition timed out"),
            Self::Occluded          => write!(f, "window is occluded"),
            Self::Outdated          => write!(f, "surface is outdated, reconfigure required"),
            Self::Lost              => write!(f, "surface lost, recreate required"),
            Self::Validation        => write!(f, "surface validation error"),
        }
    }
}

impl std::error::Error for RendererError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::DeviceRequest(e) => Some(e),
            Self::Surface(e)       => Some(e),
            _                      => None,
        }
    }
}


impl std::fmt::Debug for RendererError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}
  
impl From<wgpu::CreateSurfaceError> for RendererError {
    fn from(e: wgpu::CreateSurfaceError) -> Self { Self::Surface(e) }
}

impl From<wgpu::RequestDeviceError> for RendererError {
    fn from(e: wgpu::RequestDeviceError) -> Self { Self::DeviceRequest(e) }
}