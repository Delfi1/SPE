#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

mod engine;
mod updater;

use engine::context::{Context, ContextBuilder};
use engine::event;
use engine::event::EventHandler;
use engine::graphics::GraphicsContext;

fn main() {
    println!("Updater...");
    #[cfg(debug_assertions)]
    println!("Not release continue...");
    #[cfg(not(debug_assertions))]
    updater::update().expect("Updater error");

    let (context, event_loop) =
        ContextBuilder::new("Simple Physics Engine", "Delfi")
            .build();

    event::EventWorker::<Application>::new(context).run(event_loop);
}

pub struct Application {

}

impl EventHandler for Application {
    fn create(_ctx: &mut Context) -> Self {
        Self {}
    }
    
    fn update(&mut self, _ctx: &Context) {

    }

    fn draw(&mut self, _gfx: &mut GraphicsContext) {

    }

    fn char_input(&mut self, _ch: char) {
        print!("{_ch}");
    }

    fn on_quit(&mut self) {

    }
}
