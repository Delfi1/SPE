use vulkano::sync::GpuFuture;
use winit::dpi::PhysicalSize;
use winit::event_loop;
use winit::event_loop::EventLoop;
use super::graphics::*;
use crate::engine::config::*;
use crate::engine::input::InputContext;
use crate::engine::time::TimeContext;

pub(crate) struct Context {
    pub(crate) conf: Configuration,
    pub(crate) gfx: GraphicsContext,
    pub(crate) time: TimeContext,
    pub(crate) input: InputContext,
    pub(crate) is_running: bool
}

impl Context {
    pub(crate) fn new(conf: Configuration, event_loop: &EventLoop<()>) -> Self {

        let gfx = GraphicsContext::new(&conf, event_loop);
        let time = TimeContext::new();
        let input = InputContext::new();

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

    pub fn with_vsync(mut self, vsync: bool) -> Self {
        self.conf.window_mode.vsync = vsync;
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