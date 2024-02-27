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
use crate::engine::graphics::meshes::{CUBE_VERTICES, Mesh, Object};

pub struct Application {
    objects: Vec<Object>
}

fn main() {
    let (mut ctx, event_loop) =
        ContextBuilder::new("Simple Physics Engine", "Delfi")
            .with_fps_limit(FpsLimit::Vsync)
            .build();

    let app = Application::new(&mut ctx);

    event::run(event_loop, ctx, app)
}

impl Application {
    pub fn new(_ctx: &mut Context) -> Self {
        let mut objects = Vec::new();

        let cube_mesh = Mesh::load(CUBE_VERTICES.to_vec());
        let cube = Object::new(cube_mesh, DVec3::ZERO);

        objects.push(cube);

        Self {
            objects
        }
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
