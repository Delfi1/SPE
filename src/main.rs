use engine::context::Context;
use engine::EventHandler;

use crate::engine::context::ContextBuilder;

mod engine;

fn main() {
    let (context, event_loop) = ContextBuilder::new("SPE", "Delfi")
        .build();

    let worker = engine::Worker::<Application>::new(context);
    worker.run(event_loop);
}

pub struct Application {}

impl EventHandler for Application {
    fn setup(_context: &mut Context) -> Self {
        Self {}
    }

    fn on_quit(&self) {
        println!("Exit")
    }
}