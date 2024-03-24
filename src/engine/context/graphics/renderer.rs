use std::sync::Arc;

use vulkano::{single_pass_renderpass, VulkanLibrary};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use super::Config;

pub(super) fn create_frame_buffer(queue: Arc<Queue>, swapchain: Arc<Swapchain>, image: Arc<Image>) -> Arc<Framebuffer> {
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

pub(super) fn init_window(event_loop: &EventLoop<()>, config: &Config) -> Arc<Window> {
    let builder = WindowBuilder::new()
        .with_title(config.title.clone())
        .with_inner_size(config.size)
        .with_transparent(config.transparent)
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

/// Get current vulkan library;
pub(super) fn init_vulkan(event_loop: &EventLoop<()>) -> Arc<Instance> {
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
pub(super) fn fetch_phys_device(instance: Arc<Instance>, surface: &Arc<Surface>, extensions: &DeviceExtensions) -> (Arc<PhysicalDevice>, u32) {
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

pub(super) fn create_graphics_queue(phys_device: Arc<PhysicalDevice>, queue_family_index: u32, extensions: &DeviceExtensions) -> Arc<Queue> {
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

pub(super) fn create_swapchain(physical_device: Arc<PhysicalDevice>, surface: &Arc<Surface>, window: &Arc<Window>, device: Arc<Device>) -> (Arc<Swapchain>, Vec<Arc<Image>>) {
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