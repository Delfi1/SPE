use engine::context::Context;
use engine::context::ContextBuilder;
use engine::event::{EventHandler, EventWorker};

mod engine;
mod updater;

fn main() {
    let (context, event_loop) = ContextBuilder::new("SPE", "Delfi").build();

    EventWorker
        ::<App>::new(context).run(event_loop);
}

pub struct App {

}

impl EventHandler for App {
    fn create(_ctx: &Context) -> Self {
        Self {}
    }

    fn update(&mut self) {

    }

    fn draw(&self) {
        //println!("On draw")
    }
}