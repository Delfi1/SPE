use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::vertex_input::Vertex;

use super::{Mesh, Vertex3D};

/// Cube drawing logic;
vulkano_shaders::shader!(
    ty: "compute",
    src: r#"
        #version 460

        layout(set=0, binding=0) uniform Cube {
            vec2 resolution;
            float time;
        } cb;

        void main() {

        }
    "#
);

impl Mesh for Cube {
    fn vertex_buffer(memory_alloc: Arc<StandardMemoryAllocator>) -> Arc<Buffer> {
        let vertices = [
            Vertex3D::new(0.0, 0.0, 0.0),
            Vertex3D::new(0.0, -0.5, 0.0),
            Vertex3D::new(0.5, 0.0, 0.0)
        ];

        let sub = Buffer::from_iter(
            memory_alloc.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        ).unwrap();

        sub.buffer().clone()
    }
}