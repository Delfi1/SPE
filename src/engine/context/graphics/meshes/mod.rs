use vulkano::shader::ShaderModule;

use super::GraphicsContext;

pub struct RenderData {
    vs: ShaderModule,
    fs: ShaderModule,

}

pub trait Mesh: Sized + Send + Sync {
    fn new(gfx: &GraphicsContext) -> Self;
    fn render_data() -> RenderData;
}