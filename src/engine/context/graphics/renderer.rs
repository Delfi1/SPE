use std::sync::Arc;

use vulkano::{single_pass_renderpass, VulkanLibrary};
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::buffer::BufferUsage;
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{PresentMode, Surface, Swapchain, SwapchainCreateInfo};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use crate::engine::context::config::Config;

/// Main struct for rendering
pub(super) struct Renderer {
    pub(super) swapchain: Arc<Swapchain>,
    window: Arc<Window>,
    pub(super) queue: Arc<Queue>,
    pub(super) images: Vec<Arc<Image>>,

    pub(super) memory_alloc: Arc<StandardMemoryAllocator>,
    pub(super) buffer_alloc: Arc<StandardCommandBufferAllocator>,
    pub(super) descriptor_alloc: Arc<StandardDescriptorSetAllocator>
}

impl Renderer {
    pub(super) fn new(window: Arc<Window>, event_loop: &EventLoop<()>) -> Self {
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
            Default::default(),
        ));

        let descriptor_alloc = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        Self {
            swapchain,
            queue,
            images,
            window,

            memory_alloc,
            buffer_alloc,
            descriptor_alloc
        }
    }

    pub fn recreate_renderer(&mut self) {
        let image_extent: [u32; 2] = self.window.inner_size().into();

        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent,
                present_mode: PresentMode::FifoRelaxed,
                ..self.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");

        self.swapchain = new_swapchain;
        self.images = new_images;
    }

    pub fn frame_buffer(image: Arc<Image>, queue: Arc<Queue>, swapchain: Arc<Swapchain>) -> Arc<Framebuffer> {
        let render_pass = single_pass_renderpass!(
            queue.device().clone(),
            attachments: {
                color: {
                    format: swapchain.image_format(),
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

        let builder = match config.min_size.is_some() {
            true => builder
                .with_min_inner_size(config.min_size.unwrap()),
            _ => builder
        };

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
        },
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
    let (_, mut queues) = Device::new(
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
    let image_format = physical_device
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