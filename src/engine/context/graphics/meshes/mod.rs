use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::graphics::vertex_input::Vertex;

mod cube;

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Vertex3D {
    #[format(R32G32B32_SFLOAT)]
    pub position: [f32; 3],
}

impl Vertex3D {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            position: [x, y, z]
        }
    }
}

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Vertex2D {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}

impl Vertex2D {
    fn new(x: f32, y: f32) -> Self {
        Self {
            position: [x, y]
        }
    }
}

pub trait Mesh: Sized + Send + Sync {
    fn vertex_buffer(memory_alloc: Arc<StandardMemoryAllocator>) -> Arc<Buffer>;
}
