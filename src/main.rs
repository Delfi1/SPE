mod engine;

use std::ops::Add;
use glam::{DAffine3 as Transform, DVec3 as Vec3};
use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderPassBeginInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex as VulkanVertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::Subpass;
use vulkano::sync::GpuFuture;
use engine::context::*;
use engine::event::{self, EventHandler};
use crate::engine::config::FpsLimit;
use crate::engine::graphics::GraphicsContext;
use crate::engine::input::InputContext;
use crate::engine::shaders::ObjectShader;
use crate::engine::time::TimeContext;

#[derive(BufferContents, VulkanVertex)]
#[repr(C)]
struct Vertex {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],
}

pub struct Camera {
    transform: Transform,
}

impl Camera {
    pub(crate) fn new() -> Self {
        let mut transform = Transform::look_to_lh(Vec3::ONE, Vec3::ZERO, Vec3::Y);

        println!("{}", transform);
        Self {
            transform
        }
    }
}

pub struct Application {
    camera: Camera
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
        let camera = Camera::new();

        Self {camera}
    }
}

impl EventHandler for Application {
    fn update(&mut self, _time: &TimeContext, _input: &InputContext) {

    }

    fn draw(&mut self, _gfx: &mut GraphicsContext) {
        let queue = _gfx.graphics_queue.clone();
        let device = queue.device();


    }

    fn char_input(&mut self, ch: char) {
        println!("{}", ch);
    }

    fn on_quit(&mut self) {

    }

}
