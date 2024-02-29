use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoop;
use super::graphics::*;
use crate::engine::config::*;
use crate::engine::input::InputContext;
use crate::engine::time::TimeContext;

pub struct Context {
    pub(super) conf: Configuration,
    pub gfx: GraphicsContext,
    pub time: TimeContext,
    pub input: InputContext,
    pub(super) is_running: bool
}

impl Context {
    pub(super) fn new(conf: Configuration, event_loop: &EventLoop<()>) -> Self {
        let time = TimeContext::new();
        let input = InputContext::new();
        let gfx = GraphicsContext::new(&conf, event_loop);

        Self {
            conf,
            gfx,
            time,
            input,
            is_running: true
        }
    }
}

pub struct ContextBuilder {
    conf: Configuration
}

impl ContextBuilder {
    pub fn new(title: &str, author: &str) -> Self {
        // Create configuration;
        let conf = Configuration::new(title, author);

        Self {
            conf
        }
    }

    pub fn with_min_size(mut self, size: PhysicalSize<f32>) -> Self {
        self.conf.set_min_size(size);
        self
    }

    pub fn with_cursor_visible(mut self, visible: bool) -> Self {
        self.conf.set_cursor_visible(visible);
        self
    }

    pub fn with_size(mut self, size: PhysicalSize<f32>) -> Self {
        self.conf.set_size(size);
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self{
        self.conf.window_mode.visible = visible;
        self
    }

    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.conf.window_mode.transparent = transparent;
        self
    }

    pub fn with_fps_limit(mut self, fps_limit: FpsLimit) -> Self {
        self.conf.window_mode.fps_limit = fps_limit;
        self
    }

    pub fn with_resizable(mut self, resizable: bool) -> Self {
        self.conf.window_mode.resizable = resizable;
        self
    }

    pub fn build(mut self) -> (Context, EventLoop<()>) {
        let event_loop = EventLoop::new().expect("Event loop creation error");

        let context = Context::new(self.conf, &event_loop);

        (context, event_loop)
    }
}