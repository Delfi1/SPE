use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;
use glam::{DAffine3 as Transform, DVec3};

#[derive(Clone, BufferContents, VulkanoVertex)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 3],
}

/*
let vertices = vec![
    Vertex {position: [-1.0, -1.0, -1.0]},
    Vertex {position: [-1.0, 1.0, -1.0]},
    Vertex {position: [1.0, 1.0, -1.0]},
    Vertex {position: [1.0, -1.0, -1.0]},

    Vertex {position: [-1.0, -1.0, 1.0]},
    Vertex {position: [-1.0, 1.0, 1.0]},
    Vertex {position: [1.0, 1.0, 1.0]},
    Vertex {position: [1.0, -1.0, 1.0]},
];
*/
