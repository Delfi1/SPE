#![cfg_attr(
    not(debug_assertions),
    windows_subsystem = "windows"
)]

mod engine;
mod updater;

use engine::config::FpsLimit;
use engine::context::{Context, ContextBuilder};
use engine::event;
use engine::event::EventHandler;
use engine::graphics::GraphicsContext;
use engine::input::InputContext;
use engine::time::TimeContext;
use glam::{DAffine3 as Transform, DVec3};
use winit::keyboard::SmolStr;

pub struct Application {

}

fn main() {
    let (context, event_loop) =
        ContextBuilder::new("Simple Physics Engine", "Delfi")
            .with_fps_limit(FpsLimit::Vsync)
            .build();

    event::EventWorker::<Application>::new(context).run(event_loop);
}

impl EventHandler for Application {
    fn create(_ctx: &mut Context) -> Self {
        Self {

        }
    }
    
    fn update(&mut self, _time: &TimeContext, _input: &InputContext) {

    }

    fn draw(&mut self, _gfx: &mut GraphicsContext) {

    }

    fn button_pressed(&mut self, _btn: u32, _ch: Option<SmolStr>) {
        println!("Button \"{}\" id: {} was pressed;", _ch.unwrap_or(SmolStr::new("None")), _btn);

    }

    fn button_released(&mut self, _btn: u32) {
        println!("Button {} was released;", _btn);
    }

    fn on_quit(&mut self) {

    }

}
