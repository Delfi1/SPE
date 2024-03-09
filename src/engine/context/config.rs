use winit::dpi::PhysicalSize;

/// Configuration struct provides setup settings for engine application;
pub struct Configuration {
    // Main
    pub(in crate::engine) title: String,
    pub(in crate::engine) author: String,
    pub(super) width: i32,
    pub(super) height: i32,

    // Options
    pub(super) min_size: Option<PhysicalSize<i32>>,

    // Additional
    pub(super) visible: bool,
    pub(super) transparent: bool,
    pub(super) resizable: bool
}

impl Default for Configuration {
    fn default() -> Self {
        let title = String::from("Engine");
        let author = String::from("User");

        Self {
            title,
            author,
            width: 1280,
            height: 720,
            min_size: None,
            visible: true,
            transparent: false,
            resizable: true
        }
    }
}

impl Configuration {
    pub(super) fn new(title: &str, author: &str) -> Self {
        let title = title.to_string();
        let author = author.to_string();

        Self {
            title,
            author,
            ..Default::default()
        }
    }

    pub(super) fn actual_size(&self) -> PhysicalSize<i32> {
        PhysicalSize::new(self.width, self.height)
    }
}