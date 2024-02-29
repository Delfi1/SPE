use winit::dpi::PhysicalSize;

#[derive(Clone)]
pub(super) struct WindowSetup {
    pub(super) title: String,
    pub(super) author: String,
    icon: String
}

#[derive(Copy, Clone, PartialEq)]
pub enum FpsLimit {
    Inf,
    Vsync
}

impl FpsLimit {
    pub fn is_inf(&self) -> bool {
        self == &FpsLimit::Inf
    }

    pub fn is_vsync(&self) -> bool {
        self == &FpsLimit::Vsync
    }
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
pub(super) struct WindowMode {
    width: f32,
    height: f32,
    maximized: bool,
    pub(super) fps_limit: FpsLimit,
    pub(super) min_size: PhysicalSize<f32>,
    pub(super) borderless: bool,
    pub(super) fullscreen: bool,
    pub(super) resizable: bool,
    pub(super) cursor_visible: bool,
    pub(super) visible: bool,
    pub(super) transparent: bool
}

impl WindowMode {
    fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            ..Self::default()
        }
    }

    pub(super) fn actual_size(&self) -> PhysicalSize<f32> {
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
            min_size: PhysicalSize::new(640.0, 360.0),
            fps_limit: FpsLimit::Vsync,
            borderless: false,
            resizable: true,
            cursor_visible: true,
            visible: true,
            transparent: false
        }
    }
}

#[derive(Clone)]
pub(super) struct Configuration {
    pub(super) window_setup: WindowSetup,
    pub(super) window_mode: WindowMode
}

impl Configuration {
    pub(super) fn new(title: &str, author: &str) -> Self {
        let window_setup = WindowSetup::new(title, author);
        let window_mode = WindowMode::default();

        Self {
            window_setup,
            window_mode
        }
    }

    pub(super) fn set_cursor_visible(&mut self, visible: bool) {
        self.window_mode.cursor_visible = visible;
    }

    pub(super) fn set_size(&mut self, size: PhysicalSize<f32>) {
        self.window_mode.width = size.width;
        self.window_mode.height = size.height;
    }

    pub(super) fn set_min_size(&mut self, size: PhysicalSize<f32>) {
        self.window_mode.min_size = size;
    }
}