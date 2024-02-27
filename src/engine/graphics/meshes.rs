use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;
use glam::{DAffine3 as Transform, DVec3};

#[derive(Clone, BufferContents, VulkanoVertex)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 3],
    #[format(R32G32_SFLOAT)]
    normal: [f32; 3]
}

pub const CUBE_VERTICES: [Vertex; 8] = [
    Vertex {position: [-1.0, 1.0, 0.0], normal: [0.0, 0.0, 0.0]},
    Vertex {position: [-1.0, -1.0, 0.0], normal: [0.0, 0.0, 0.0]},
    Vertex {position: [1.0, -1.0, 0.0], normal: [0.0, 0.0, 0.0]},
    Vertex {position: [1.0, 1.0, 0.0], normal: [0.0, 0.0, 0.0]},

    Vertex {position: [-1.0, 1.0, -1.0], normal: [0.0, 0.0, 0.0]},
    Vertex {position: [-1.0, -1.0, -1.0], normal: [0.0, 0.0, 0.0]},
    Vertex {position: [1.0, -1.0, -1.0], normal: [0.0, 0.0, 0.0]},
    Vertex {position: [1.0, 1.0, -1.0], normal: [0.0, 0.0, 0.0]},
];

pub struct Mesh {
    vertices: Vec<Vertex>
}

impl Mesh {
    pub fn load(vertices: Vec<Vertex>) -> Self {
        Self {vertices}
    }
}

pub struct Object {
    mesh: Mesh,
    transform: Transform,
    scale: f64
}

impl Object {
    pub fn new(mesh: Mesh, position: DVec3) -> Self {
        let transform = Transform::from_translation(position);

        Self {
            mesh,
            transform,
            scale: 1.0
        }
    }

    pub fn with_scale(mut self, scale: f64) -> Self {
        self.scale = scale;
        self
    }
}