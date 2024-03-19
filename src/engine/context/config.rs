use winit::dpi::PhysicalSize;

pub struct Config {
    pub(super) title: String,
    pub(super) author: String,
    pub(super) size: PhysicalSize<i32>,
    pub(super) min_size: Option<PhysicalSize<i32>>,
    pub(super) visible: bool,
    pub(super) transparent: bool,
}

impl Config {
    pub(super) fn new(title: &str, author: &str) -> Self {
        Self {
            title: title.to_string(),
            author: author.to_string(),
            ..Self::default()
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            title: "SPE".to_string(),
            author: "Delfi".to_string(),
            size: (1280, 720).into(),
            min_size: None,
            visible: true,
            transparent: false,
        }
    }
}