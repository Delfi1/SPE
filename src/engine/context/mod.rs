use winit::event_loop::EventLoop;

use config::Config;
use graphics::GraphicsContext;
use input::InputContext;
use time::TimeContext;

mod input;
mod time;
mod graphics;
mod config;

pub struct Context {
    pub(super) title: String,
    pub(super) author: String,
    pub(super) input: InputContext,
    pub(super) time: TimeContext,
    pub(super) graphics: GraphicsContext,
}

impl Context {
    pub fn new() -> (Self, EventLoop<()>) {
        let builder = ContextBuilder::default();
        builder.build()
    }
}

#[derive(Default)]
pub struct ContextBuilder {
    config: Config,
}

impl ContextBuilder {
    pub fn new(title: &str, author: &str) -> Self {
        Self { config: Config::new(title, author) }
    }

    pub fn with_transparent(mut self, transparent: bool) -> Self {
        self.config.transparent = transparent;
        self
    }

    pub fn with_visible(mut self, visible: bool) -> Self {
        self.config.visible = visible;
        self
    }

    pub fn with_min_size(mut self, min: Option<(i32, i32)>) -> Self {
        self.config.min_size = match min {
            Some(size) => Some(size.into()),
            None => None
        };

        self
    }

    pub fn with_size(mut self, width: i32, height: i32) -> Self {
        self.config.size = (width, height).into();
        self
    }

    pub fn build(self) -> (Context, EventLoop<()>) {
        let event_loop = EventLoop::new().unwrap();
        let title = self.config.title.clone();
        let author = self.config.author.clone();

        let time = TimeContext::new();
        let input = InputContext::new();
        let graphics = GraphicsContext::new(&self.config, &event_loop);

        (Context { title, author, time, input, graphics }, event_loop)
    }
}
