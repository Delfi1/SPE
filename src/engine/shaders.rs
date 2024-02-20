use std::sync::Arc;
use vulkano::device::Device;
use vulkano::shader::{EntryPoint, ShaderModule, ShaderStage};
use vulkano::{Validated, VulkanError};
use vulkano::buffer::BufferContents;
use vulkano::pipeline::graphics::vertex_input::Vertex as VulkanoVertex;

#[derive(BufferContents, VulkanoVertex)]
#[repr(C)]
pub struct Vertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

mod vs {
    vulkano_shaders::shader!{
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec2 position;

            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
            }
        ",
    }
}

mod fs {
    vulkano_shaders::shader!{
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(1.0, 0.0, 0.0, 1.0);
            }
        ",
    }
}

pub struct ObjectShader {
    pub vs: Arc<ShaderModule>,
    pub fs: Arc<ShaderModule>
}

impl ObjectShader {
    pub fn load(device: Arc<Device>) -> Result<Self, Validated<VulkanError>> {
        Ok(Self {
            vs: vs::load(device.clone())?,
            fs: fs::load(device.clone())?
        })
    }
}