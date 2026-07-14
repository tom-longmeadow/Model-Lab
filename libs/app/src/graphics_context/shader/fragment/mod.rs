#[derive(Clone, Copy)]
pub enum FragmentFunction {
    Circular,
    Passthrough,
}

impl FragmentFunction {
    pub fn source(&self) -> &'static str {
        match self {
            FragmentFunction::Circular => include_str!("circular.wgsl"),
            FragmentFunction::Passthrough => include_str!("passthrough.wgsl"),
        }
    }

    /// The name of the function inside the .wgsl file.
    pub fn fn_name(&self) -> &'static str {
        match self {
            FragmentFunction::Circular => "fs_circular",
            FragmentFunction::Passthrough => "fs_passthrough",
        }
    }
}