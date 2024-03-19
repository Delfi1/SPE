use std::sync::Arc;
use std::thread;

use vulkano::{Validated, VulkanError};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter};
use vulkano::swapchain::{acquire_next_image, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use winit::event_loop::EventLoop;
use winit::window::Window;

use renderer::Renderer;

use crate::engine::context::graphics::renderer::MyVertex;

use super::Config;

mod renderer;
mod meshes;

/// Graphics context;
/// Window, surface,
/// Swapchain, draw_data, image;
pub struct GraphicsContext {
    window: Arc<Window>,
    outdated: bool,
    image_index: usize,
    renderer: Arc<Renderer>,
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

    pub fn window(&self) -> Arc<Window> {
        self.window.clone()
    }

    pub(in crate::engine) fn acquire(&mut self) -> Result<Box<dyn GpuFuture + Send + Sync>, VulkanError> {
        if self.outdated {
            self.renderer = self.renderer.recreate_renderer();
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

    pub fn resized(&mut self) {
        self.outdated = true;
    }

    pub fn redraw(&mut self, acquired: Box<dyn GpuFuture + Sync + Send>) {
        let window = self.window.clone();
        let renderer = self.renderer.clone();
        let image_index = self.image_index;

        thread::spawn(move || {
            let mut command_buffer = AutoCommandBufferBuilder::primary(
                &renderer.buffer_alloc,
                renderer.queue.queue_family_index(),
                CommandBufferUsage::MultipleSubmit
            ).unwrap();

            let vertex1 = MyVertex { position: [-0.5, -0.5] };
            let vertex2 = MyVertex { position: [ 0.0,  0.5] };
            let vertex3 = MyVertex { position: [ 0.5, -0.25] };

            let vertex_buffer = Buffer::from_iter(
                renderer.memory_alloc.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::VERTEX_BUFFER,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                        | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                vec![vertex1, vertex2, vertex3],
            ).unwrap();

            let frame_buffer = renderer.frame_buffer(image_index);
            let pipeline = renderer.create_pipeline(frame_buffer.render_pass().clone());

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
                .bind_vertex_buffers(0, vertex_buffer.clone())
                .unwrap()
                .draw(
                    3, 1, 0, 0,
                )
                .unwrap()
                .end_render_pass(
                    SubpassEndInfo::default()
                ).unwrap();

            let after_future =
                command_buffer.build().unwrap()
                    .execute(renderer.queue.clone()).unwrap()
                    .join(acquired).boxed();

            let future = after_future
                .then_swapchain_present(
                    renderer.queue.clone(),
                    SwapchainPresentInfo::swapchain_image_index(
                        renderer.swapchain.clone(),
                        image_index as u32,
                    ),
                )
                .then_signal_fence_and_flush();

            match future.map_err(Validated::unwrap) {
                Ok(future) => {
                    future.wait(None).unwrap_or_else(|e| println!("{e}"));
                }
                Err(VulkanError::OutOfDate) => ()
                Err(e) => {
                    println!("failed to flush future: {e}");
                }
            }

            window.request_redraw();
        });
    }

}