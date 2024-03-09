use winit::event_loop::EventLoop;

use config::Configuration;
use graphics::GraphicsContext;
use input::InputContext;
use time::TimeContext;

mod config;

mod graphics;
mod input;
mod time;

/// Main context object;
pub struct Context {
    pub(super) conf: Configuration,
    pub(super) graphics: GraphicsContext,
    pub(super) input: InputContext,
    pub(super) time: TimeContext
}

impl Context {
    #[inline]
    pub(super) fn new(conf: Configuration, event_loop: &EventLoop<()>) -> Self {
        let graphics = GraphicsContext::new(&conf, event_loop);
        let input = InputContext::new();
        let time = TimeContext::new();
        graphics.window().request_redraw();

        Self {
            conf,
            graphics,
            input,
            time
        }
    }
}

/// Context builder helps with window initialization;
pub struct ContextBuilder {
    conf: Configuration,
    event_loop: EventLoop<()>
}

impl ContextBuilder {
    #[inline]
    pub fn new(title: &str, author: &str) -> Self {
        Self {
            conf: Configuration::new(title, author),
            event_loop: EventLoop::new().expect("Setup error")
        }
    }

    /// Set window init visible;
    #[inline]
    pub fn with_visible(mut self, visible: bool) -> Self {
        self.conf.visible = visible;
        self
    }

    /// Set window init size;
    #[inline]
    pub fn with_size(mut self, width: i32, height: i32) -> Self {
        self.conf.width = width;
        self.conf.height = height;
        self
    }

    /// Finish preparing and create context
    #[inline]
    pub fn build(self) -> (Context, EventLoop<()>) {
        let context = Context::new(self.conf, &self.event_loop);

        (context, self.event_loop)
    }
}