mod engine;

use engine::config::FpsLimit;
use engine::context::{Context, ContextBuilder};
use engine::event;
use engine::event::EventHandler;
use engine::graphics::GraphicsContext;
use engine::input::InputContext;
use engine::time::TimeContext;
use engine::updater::update;
use crate::engine::updater::check_updates;

pub struct Application {

}

fn main() {
    let (mut ctx, event_loop) =
        ContextBuilder::new("Simple Physics Engine", "Delfi")
            .with_fps_limit(FpsLimit::Vsync)
            .with_visible(false)
            .build();

    let app = Application::new(&mut ctx);

    event::run(event_loop, ctx, app)
}

impl Application {
    pub fn new(_ctx: &mut Context) -> Self {
        let raw_releases = check_updates();
        if raw_releases.is_ok() {
            let releases = raw_releases.unwrap();

            update(releases).expect("Update error");
        } else {
            println!("Not new versions was found")
        }
        _ctx.gfx.window().set_visible(true);
        Self {}
    }
}

impl EventHandler for Application {
    fn update(&mut self, _time: &TimeContext, _input: &InputContext) {

    }

    fn draw(&mut self, _gfx: &mut GraphicsContext) {

    }

    fn char_input(&mut self, ch: char) {
        println!("{}", ch);
    }

    fn on_quit(&mut self) {

    }

}
