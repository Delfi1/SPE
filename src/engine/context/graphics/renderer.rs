use std::sync::Arc;

use vulkano::{single_pass_renderpass, VulkanLibrary};
use vulkano::buffer::BufferContents;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass};
use vulkano::swapchain::{PresentMode, Surface, Swapchain, SwapchainCreateInfo};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::engine::context::config::Config;

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

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct MyVertex {
    #[format(R32G32_SFLOAT)]
    pub position: [f32; 2],
}

/// Main struct for rendering
pub(super) struct Renderer {
    pub(super) swapchain: Arc<Swapchain>,
    window: Arc<Window>,
    pub(super) queue: Arc<Queue>,
    pub(super) images: Vec<Arc<Image>>,

    pub(super) memory_alloc: Arc<StandardMemoryAllocator>,
    pub(super) buffer_alloc: Arc<StandardCommandBufferAllocator>
}

impl Renderer {
    pub(super) fn new(window: Arc<Window>, event_loop: &EventLoop<()>) -> Arc<Self> {
        let instance = init_vulkan(event_loop);

        let extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("Surface create error");

        let (phys_device, queue_index) =
            fetch_phys_device(instance.clone(), &surface, &extensions);

        let queue = create_graphics_queue(phys_device.clone(), queue_index, &extensions);
        let device = queue.device();

        let (swapchain, images) =
            create_swapchain(phys_device.clone(), &surface, &window, device.clone());

        let memory_alloc = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let buffer_alloc = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));

        Arc::new(Self {
            swapchain,
            queue,
            images,
            window,

            memory_alloc,
            buffer_alloc
        })
    }

    pub fn recreate_renderer(&self) -> Arc<Self> {
        let window = self.window.clone();
        let queue = self.queue.clone();
        let memory_alloc = self.memory_alloc.clone();
        let buffer_alloc = self.buffer_alloc.clone();

        let image_extent: [u32; 2] = self.window.inner_size().into();

        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent,
                present_mode: PresentMode::FifoRelaxed,
                ..self.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");

        Arc::new(
            Self {
                swapchain: new_swapchain,
                queue,
                images: new_images,
                window,

                memory_alloc,
                buffer_alloc
            }
        )
    }

    pub fn create_pipeline(&self, render_pass: Arc<RenderPass>) -> Arc<GraphicsPipeline> {
        let device = self.queue.device();

        let vs = vs::load(device.clone()).expect("failed to create shader module");
        let fs = fs::load(device.clone()).expect("failed to create shader module");

        let vertex_shader = vs.entry_point("main").unwrap();
        let fragment_shader = fs.entry_point("main").unwrap();

        let vertex_input_state = MyVertex::per_vertex()
            .definition(&vertex_shader.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vertex_shader),
            PipelineShaderStageCreateInfo::new(fragment_shader),
        ];

        let viewport = Viewport {
            offset: [0.0, 0.0],
            extent: self.window.inner_size().into(),
            depth_range: 0.0..=1.0,
        };

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        ).unwrap();

        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                viewport_state: Some(ViewportState {
                    viewports: [viewport].into_iter().collect(),
                    ..Default::default()
                }),
                input_assembly_state: Some(InputAssemblyState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                // This graphics pipeline object concerns the first pass of the render pass.
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        ).unwrap()
    }

    pub fn frame_buffer(&self, image_index: usize) -> Arc<Framebuffer> {
        let render_pass = single_pass_renderpass!(
            self.queue.device().clone(),
            attachments: {
                color: {
                    format: self.swapchain.image_format(),
                    samples: 1,
                    load_op: Clear,
                    store_op: Store,
                },
            },
            pass: {
                color: [color],
                depth_stencil: {},
            },
        ).expect("Render pass init error");

        let image = self.images[image_index].clone();
        let view = ImageView::new_default(image).unwrap();
        Framebuffer::new(
            render_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            },
        ).unwrap()
    }

    pub fn init_window(event_loop: &EventLoop<()>, config: &Config) -> Arc<Window> {
        let builder = WindowBuilder::new()
            .with_title(config.title.clone())
            .with_inner_size(config.size)
            .with_transparent(config.transparent)
            .with_visible(false)
            .with_active(true);

        Arc::new(
            builder.build(event_loop).unwrap()
        )
    }
}

/// Get current vulkan library;
fn init_vulkan(event_loop: &EventLoop<()>) -> Arc<Instance> {
    let library = VulkanLibrary::new().expect("Vulkano library not found");

    Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: Surface::required_extensions(event_loop),
            ..Default::default()
        }
    ).unwrap()
}

/// Returns physical device and queue family index
fn fetch_phys_device(instance: Arc<Instance>, surface: &Arc<Surface>, extensions: &DeviceExtensions) -> (Arc<PhysicalDevice>, u32) {
    instance
        .enumerate_physical_devices()
        .expect("could not enumerate devices")
        .filter(|p| p.supported_extensions().contains(extensions))
        .filter_map(|p| {
            p.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && p.surface_support(i as u32, surface).unwrap_or(false)
                })
                .map(|q| (p, q as u32))
        })
        .min_by_key(|(p, _)| match p.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("no device available")
}

fn create_graphics_queue(phys_device: Arc<PhysicalDevice>, queue_family_index: u32, extensions: &DeviceExtensions) -> Arc<Queue> {
    let (device, mut queues) = Device::new(
        phys_device.clone(),
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: *extensions,
            ..Default::default()
        },
    ).expect("failed to create device");

    queues.next().unwrap()
}

fn create_swapchain(physical_device: Arc<PhysicalDevice>, surface: &Arc<Surface>, window: &Arc<Window>, device: Arc<Device>) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
    let caps = physical_device
        .surface_capabilities(surface, Default::default())
        .expect("failed to get surface capabilities");

    let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
    let image_format =  physical_device
        .surface_formats(&surface, Default::default())
        .unwrap()[0]
        .0;

    Swapchain::new(
        device.clone(),
        surface.clone(),
        SwapchainCreateInfo {
            min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
            image_extent: window.inner_size().into(),
            image_format,
            image_usage: ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
            composite_alpha,
            ..Default::default()
        },
    ).unwrap()
}