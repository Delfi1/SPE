use std::sync::Arc;
use vulkano::buffer::BufferContents;
use vulkano::device::Device;
use super::MyVertex;

mod vs {
    vulkano_shaders::shader! {
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
    vulkano_shaders::shader! {
        ty: "fragment",
        src: "
            #version 460

            layout(location = 0) out vec4 f_color;

            void main() {
                f_color = vec4(0.0, 0.0, 0.7, 1.0);
            }
        ",
    }
}

pub struct Triangle {

}
