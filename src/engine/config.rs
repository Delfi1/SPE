use winit::dpi::PhysicalSize;
use winit::window::Fullscreen;

#[derive(Clone)]
pub struct WindowSetup {
    pub(crate) title: String,
    pub(crate) author: String,
    icon: String
}

impl WindowSetup {
    fn new(title: &str, author: &str) -> Self {
        let title = title.to_string().to_owned();
        let author = author.to_string().to_owned();

        Self {
            title,
            author,
            ..Self::default()
        }
    }
}

impl Default for WindowSetup {
    fn default() -> Self {
        let title = "Engine".to_string();
        let author = "User".to_string();
        let icon = String::new();

        Self {title, author, icon}
    }
}

#[derive(Clone)]
pub struct WindowMode {
    width: f32,
    height: f32,
    maximized: bool,
    pub(crate) vsync: bool,
    pub(crate) borderless: bool,
    pub(crate) fullscreen: bool,
    pub(crate) resizable: bool,
    pub(crate) visible: bool,
    pub(crate) transparent: bool
}

impl WindowMode {
    fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            ..Self::default()
        }
    }

    pub(crate) fn actual_size(&self) -> PhysicalSize<f32> {
        PhysicalSize::new(self.width, self.height)
    }
}

impl Default for WindowMode {
    fn default() -> Self {
        Self {
            width: 1280.0,
            height: 720.0,
            maximized: false,
            fullscreen: false,
            vsync: true,
            borderless: false,
            resizable: true,
            visible: true,
            transparent: false
        }
    }
}

#[derive(Clone)]
pub struct Configuration {
    pub(crate) window_setup: WindowSetup,
    pub(crate) window_mode: WindowMode
}

impl Configuration {
    pub(crate) fn new(title: &str, author: &str) -> Self {
        let window_setup = WindowSetup::new(title, author);
        let window_mode = WindowMode::default();

        Self {
            window_setup,
            window_mode
        }
    }

    pub(crate) fn set_size(&mut self, size: PhysicalSize<f32>) {
        self.window_mode.width = size.width;
        self.window_mode.height = size.height;
    }
}