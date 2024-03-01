use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;
use glam::{DAffine3 as Transform, DVec3};
use vulkano::command_buffer::{AutoCommandBufferBuilder, PrimaryCommandBufferAbstract, SecondaryCommandBufferAbstract};

#[derive(Clone, BufferContents, VulkanoVertex)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 3],
}

pub struct RenderData {
    vertices: Vec<Vertex>,
    //color
    //
}

trait Object: Sized + 'static {
    fn new() -> Self;
    fn update(&mut self);
    fn draw(&self) -> impl PrimaryCommandBufferAbstract;
}

pub struct Cube {

}
