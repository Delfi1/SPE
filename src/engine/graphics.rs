use std::sync::Arc;
use vulkano::swapchain::{Surface, Swapchain, SwapchainCreateInfo};
use vulkano::{Validated, VulkanError, VulkanLibrary};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};
use crate::engine::config::Configuration;

#[derive()]
pub struct Renderer {
    queue: Arc<Queue>,
    surface: Arc<Surface>,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>
}

impl Renderer {
    pub(self) fn new(window: Arc<Window>, event_loop: &EventLoop<()>) -> Result<Arc<Self>, Validated<VulkanError>> {
        let library = VulkanLibrary::new().unwrap();

        let required_extensions = Surface::required_extensions(event_loop);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..InstanceCreateInfo::default()
            }
        ).expect("Can't create instance");

        let surface = Surface::from_window(
            instance.clone(),
            window.clone()
        ).expect("Can't create surface");

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let physical_device =
            Self::get_physical_device(instance.clone(), surface.clone(), &device_extensions).unwrap();

        let queue = Self::get_queue(physical_device.clone(), &device_extensions);
        let device = queue.device();

        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let dimensions = window.inner_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format =
            physical_device
                .surface_formats(&surface, Default::default())
                .unwrap()[0]
                .0;

        let (mut swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha,
                ..Default::default()
            }
        ).unwrap();

        Ok(
            Arc::new(Self {
                queue,
                surface,
                swapchain,
                images
            })
        )
    }

    pub(self) fn get_physical_device(
        instance: Arc<Instance>,
        surface: Arc<Surface>,
        device_extensions: &DeviceExtensions
    ) -> Result<Arc<PhysicalDevice>, Validated<VulkanError>> {
        let (physical_device, _) = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                p.queue_family_properties()
                    .iter()
                    .enumerate()
                    .position(|(i, q)| {
                        q.queue_flags.contains(QueueFlags::GRAPHICS)
                            && p.surface_support(i as u32, &surface).unwrap_or(false)
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
            .expect("no device available");

       Ok(physical_device)
    }

    pub(self) fn get_queue(
        physical_device: Arc<PhysicalDevice>,
        device_extensions: &DeviceExtensions
    ) -> Arc<Queue>{
        let queue_family_index = physical_device
            .queue_family_properties()
            .iter()
            .enumerate()
            .position(|(_queue_family_index, queue_family_properties)| {
                queue_family_properties.queue_flags.contains(QueueFlags::GRAPHICS)
            })
            .expect("couldn't find a graphical queue family") as u32;

        let (_, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                enabled_extensions: *device_extensions,
                ..Default::default()
            },
        ).expect("failed to create device");

        queues.next().expect("queue not found")
    }

    pub(self) fn create_window(conf: &Configuration, event_loop: &EventLoop<()>) -> Arc<Window> {
        #[cfg(target_os = "windows")]
            let mut window_builder = {
            use winit::platform::windows::WindowBuilderExtWindows;
            WindowBuilder::new().with_drag_and_drop(false)
        };

        #[cfg(not(target_os = "windows"))]
            let mut window_builder = winit::window::WindowBuilder::new();

        let window_builder = window_builder
            .with_title(conf.window_setup.title.clone())
            .with_resizable(conf.window_mode.resizable)
            .with_visible(conf.window_mode.visible)
            .with_transparent(conf.window_mode.transparent)
            .with_inner_size(conf.window_mode.actual_size());

        let window = Arc::new(
            window_builder
                .build(event_loop).unwrap()
        );

        window
    }
}

pub struct GraphicsContext {
    window: Arc<Window>,
    renderer: Arc<Renderer>
}

impl GraphicsContext {
    pub(super) fn new(conf: &Configuration, event_loop: &EventLoop<()>) -> Self {
        let window = Renderer::create_window(conf, event_loop);

        let renderer = Renderer::new(window.clone(), event_loop).unwrap();

        GraphicsContext {
            window,
            renderer
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub(super) fn resize(&self) {
        //Todo
    }
}