use std::sync::Arc;
use std::thread;
use std::time::Duration;

use vulkano::{Validated, VulkanError};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::Subpass;
use vulkano::swapchain::{acquire_next_image, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use winit::event_loop::EventLoop;
use winit::window::Window;

use renderer::Renderer;

use super::Config;

mod renderer;
mod meshes;
use meshes::MyVertex;

/// Graphics context;
/// Window, surface,
/// Swapchain, draw_data, image;
pub struct GraphicsContext {
    window: Arc<Window>,
    outdated: bool,
    image_index: usize,
    renderer: Renderer,
}

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
    vulkano_shaders::shader!{
        ty: "fragment",
        src: r#"
            #version 460

            layout(set=0, binding=0) uniform InData {
                vec2 resolution;
                float time;
            } ub;
            vec2 fragCoord = gl_FragCoord.xy;

            layout(location=0) out vec4 fragColor;

            void main()
            {
                // Normalized pixel coordinates (from 0 to 1)
                vec2 uv = fragCoord / ub.resolution.xy;

                // Time varying pixel color
                vec3 col = 0.5 + 0.5*cos(ub.time + uv.xyx + vec3(0,2,4));

                // Output to screen
                fragColor = vec4(col,1.0);
            }
        "#,
    }
}

impl GraphicsContext {
    pub(super) fn new(config: &Config, event_loop: &EventLoop<()>) -> Self {
        let window = Renderer::init_window(event_loop, config);
        let renderer = Renderer::new(window.clone(), event_loop);
        window.set_visible(config.visible);

        Self {
            window,
            outdated: true,
            image_index: 0,
            renderer,
        }
    }

    pub(in crate::engine) fn resized(&mut self) {
        self.outdated = true;
    }

    pub(in crate::engine) fn acquire(&mut self) -> Result<Box<dyn GpuFuture + Send + Sync>, VulkanError> {
        if self.outdated {
            self.renderer.recreate_renderer();
            self.outdated = false;
        }

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.renderer.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.outdated = true;
                    return Err(VulkanError::OutOfDate);
                }
                Err(_e) => panic!("failed to acquire next image: {_e}"),
            };

        if suboptimal {
            self.outdated = true;
        }

        self.image_index = image_index as usize;

        Ok(acquire_future.boxed_send_sync())
    }

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub fn redraw(&mut self, acquired: Box<dyn GpuFuture + Send + Sync>, init_time: Duration) {
        let window = self.window.clone();
        let queue = self.renderer.queue.clone();
        let swapchain = self.renderer.swapchain.clone();

        let buffer_alloc = self.renderer.buffer_alloc.clone();
        let memory_alloc = self.renderer.memory_alloc.clone();
        let descriptor_alloc = self.renderer.descriptor_alloc.clone();
        let image_index = self.image_index;
        let image = self.renderer.images[self.image_index].clone();

        thread::spawn(move || {
            let mut command_buffer = AutoCommandBufferBuilder::primary(
                &buffer_alloc,
                queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit
            ).unwrap();

            let vertex1 = MyVertex { position: [-1.0, -1.0] };
            let vertex2 = MyVertex { position: [ 1.0,  -1.0] };
            let vertex3 = MyVertex { position: [ 1.0, 1.0] };
            let vertex4 = MyVertex { position: [ 1.0, 1.0] };
            let vertex5 = MyVertex { position: [ -1.0, 1.0] };
            let vertex6 = MyVertex { position: [ -1.0, -1.0] };

            let vertex_buffer = Buffer::from_iter(
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
                vec![vertex1, vertex2, vertex3, vertex4, vertex5, vertex6],
            ).unwrap();

            let frame_buffer = Renderer::frame_buffer(image, queue.clone(), swapchain.clone());
            let pipeline = {
                let device = queue.device();
                let vs = vs::load(device.clone()).unwrap();
                let fs = fs::load(device.clone()).unwrap();

                let vertex_shader = vs.entry_point("main").unwrap();
                let fragment_shader = fs.entry_point("main").unwrap();

                let vertex_input_state = MyVertex::per_vertex()
                    .definition(&vertex_shader.info().input_interface)
                    .unwrap();

                let stages = [
                    PipelineShaderStageCreateInfo::new(vertex_shader),
                    PipelineShaderStageCreateInfo::new(fragment_shader),
                ];

                let layout = PipelineLayout::new(
                    queue.device().clone(),
                    PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                        .into_pipeline_layout_create_info(queue.device().clone())
                        .unwrap(),
                ).unwrap();

                let render_pass = frame_buffer.render_pass();
                let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

                let viewport = Viewport {
                    offset: [0.0, 0.0],
                    extent: window.inner_size().into(),
                    depth_range: 0.0..=1.0,
                };

                GraphicsPipeline::new(
                    queue.device().clone(),
                    None,
                    GraphicsPipelineCreateInfo {
                        stages: stages.into_iter().collect(),
                        vertex_input_state: Some(vertex_input_state),
                        input_assembly_state: Some(InputAssemblyState::default()),
                        viewport_state: Some(ViewportState {
                            viewports: [viewport].into_iter().collect(),
                            ..Default::default()
                        }),
                        rasterization_state: Some(RasterizationState::default()),
                        multisample_state: Some(MultisampleState::default()),
                        color_blend_state: Some(ColorBlendState::with_attachment_states(
                            subpass.num_color_attachments(),
                            ColorBlendAttachmentState::default(),
                        )),
                        subpass: Some(subpass.into()),
                        ..GraphicsPipelineCreateInfo::layout(layout)
                    },
                ).unwrap()
            };

            let uniform_buffer = SubbufferAllocator::new(
                memory_alloc.clone(),
                SubbufferAllocatorCreateInfo {
                    buffer_usage: BufferUsage::UNIFORM_BUFFER,
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
            );

            let resolution: [f32; 2] = window.inner_size().into();

            let subbuffer = {
                let uniform_data = fs::InData {
                    resolution,
                    time: init_time.as_secs_f32()
                };

                let subbuffer = uniform_buffer.allocate_sized().unwrap();
                *subbuffer.write().unwrap() = uniform_data;

                subbuffer
            };

            let layout = pipeline.layout().set_layouts().get(0).unwrap();
            let set = PersistentDescriptorSet::new(
                &descriptor_alloc,
                layout.clone(),
                [WriteDescriptorSet::buffer(0, subbuffer)],
                [],
            ).unwrap();

            command_buffer
                .begin_render_pass(
                    RenderPassBeginInfo {
                        clear_values: vec![Some([22.0/255.0, 22.0/255.0, 29.0/255.0, 1.0].into())],
                        ..RenderPassBeginInfo::framebuffer(frame_buffer)
                    },
                    SubpassBeginInfo::default()
                ).unwrap()
                .bind_pipeline_graphics(pipeline.clone())
                .unwrap()
                .bind_descriptor_sets(
                    PipelineBindPoint::Graphics,
                    pipeline.layout().clone(),
                    0,
                    set,
                )
                .unwrap()
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .unwrap()
                .draw(
                    6, 1, 0, 0,
                )
                .unwrap()
                .end_render_pass(
                    SubpassEndInfo::default()
                ).unwrap();

            let after_future =
                command_buffer.build().unwrap()
                    .execute(queue.clone()).unwrap()
                    .join(acquired).boxed();

            let future = after_future
                .then_swapchain_present(
                    queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(
                        swapchain.clone(),
                        image_index as u32,
                    ),
                )
                .then_signal_fence_and_flush();

            match future.map_err(Validated::unwrap) {
                Ok(future) => {
                    future.wait(None).unwrap_or_else(|e| println!("{e}"));
                }
                Err(VulkanError::OutOfDate) => (),
                Err(e) => {
                    println!("failed to flush future: {e}");
                }
            }

            window.request_redraw();
        });
    }
}