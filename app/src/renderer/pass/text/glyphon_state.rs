 
 

pub struct GlyphonGroup {
    pub buffer: glyphon::Buffer,
    pub left: f32,
    pub top: f32,
    pub buf_width: f32,
    pub color: [u8; 4],
}

pub struct GlyphonState {
    pub width: u32,
    pub height: u32,
    pub font_system: glyphon::FontSystem,
    pub swash_cache: glyphon::SwashCache,
    pub viewport: glyphon::Viewport,
    pub atlas: glyphon::TextAtlas,
    pub renderer: glyphon::TextRenderer,
    pub groups: Vec<GlyphonGroup>,  
}