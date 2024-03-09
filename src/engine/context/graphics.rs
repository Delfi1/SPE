use std::sync::{Arc, Mutex};
use std::thread;

use vulkano::{single_pass_renderpass, Validated, VulkanError, VulkanLibrary};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, DeviceOwned, Queue, QueueCreateInfo, QueueFlags};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::format::Format;
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use vulkano::swapchain::{acquire_next_image, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use winit::error::OsError;
use winit::event_loop::EventLoop;
use winit::window::{Window, WindowBuilder};

use super::Configuration;

pub struct Renderer {
    outdated: bool,
    swapchain: Arc<Swapchain>,
    image_index: u32,
    images: Vec<Arc<Image>>,

    window: Arc<Window>,
    surface: Arc<Surface>,
    queue: Arc<Queue>
}

impl Renderer {
    fn new(event_loop: &EventLoop<()>, window: Arc<Window>) -> Arc<Mutex<Self>> {
        let library = VulkanLibrary::new().expect("Library not found");

        let required_extensions = Surface::required_extensions(event_loop);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..InstanceCreateInfo::default()
            }
        ).expect("Instance create error");

        let surface = Surface::from_window(
            instance.clone(),
            window.clone()
        ).unwrap();

        let device_extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let physical_device =
            Self::get_physical_device(instance.clone(), surface.clone(), &device_extensions).unwrap();

        let queue = Self::create_queue(physical_device.clone(), &device_extensions);
        let device = queue.device();

        let caps = physical_device.clone()
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let dimensions = window.inner_size();
        let composite_alpha = caps.supported_composite_alpha.into_iter().next().unwrap();
        let image_format = physical_device.clone()
                .surface_formats(&surface, Default::default())
                .unwrap()[0].0;

        let (swapchain, images) = Swapchain::new(
            device.clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1, // How many buffers to use in the swapchain
                image_format,
                image_extent: dimensions.into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT, // What the images are going to be used for
                composite_alpha,
                ..Default::default()
            }
        ).expect("Swapchain create error");

        Arc::new(Mutex::new(Self {
            outdated: false,
            swapchain,
            images,
            image_index: 0,
            queue,
            window,
            surface
        }))
    }

    fn recreate_swapchain_and_views(&mut self) {
        let image_extent: [u32; 2] = self.window.inner_size().into();

        if image_extent.contains(&0) {
            return;
        }

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

        self.outdated = false;
    }

    pub(self) fn swapchain_format(&self) -> Format {
        self.swapchain.image_format()
    }

    pub(self) fn swapchain_view(&self) -> Arc<ImageView> {
        ImageView::new_default(self.images[self.image_index as usize].clone())
            .expect("View create error")
    }

    fn create_queue(
        physical_device: Arc<PhysicalDevice>,
        extensions: &DeviceExtensions
    ) -> Arc<Queue> {
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
                enabled_extensions: *extensions,
                ..Default::default()
            },
        ).expect("failed to create device");

        queues.next().expect("queue not found")
    }

    fn get_physical_device(
        instance: Arc<Instance>,
        surface: Arc<Surface>,
        extensions: &DeviceExtensions
    ) -> Result<Arc<PhysicalDevice>, Validated<VulkanError>> {
        let (physical_device, _) = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            .filter(|p| p.supported_extensions().contains(&extensions))
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

    fn present(&mut self, after_future: Box<dyn GpuFuture + 'static>) {
        let future = after_future
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    self.image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(future) => {
                future.wait(None).unwrap_or_else(|e| println!("{e}"));
            }
            Err(VulkanError::OutOfDate) => {
                self.outdated = true;
            }
            Err(e) => {
                println!("failed to flush future: {e}");
            }
        }
    }

    pub(crate) fn acquire(&mut self) -> Result<Box<dyn GpuFuture + Send + Sync>, VulkanError> {
        if self.outdated {
            self.recreate_swapchain_and_views();
        }

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None)
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

        self.image_index = image_index;

        Ok(acquire_future.boxed_send_sync())
    }
}

/// Graphics context
/// Window, surface
/// Swapchain, draw_data, image
pub struct GraphicsContext {
    window: Arc<Window>,
    renderer: Arc<Mutex<Renderer>>
}

impl GraphicsContext {
    pub(super) fn new(conf: &Configuration, event_loop: &EventLoop<()>) -> Self {
        let window = Self::create_window(conf, event_loop)
            .expect("Create window error");

        let renderer = Renderer::new(event_loop, window.clone());

        window.set_visible(conf.visible);

        Self { window, renderer }
    }

    #[inline]
    pub(in crate::engine) fn set_outdated(&mut self) {
        let mut renderer = self.renderer.lock().unwrap();

        renderer.outdated = true;
    }

    #[inline]
    pub (in crate::engine) fn acquire(&mut self) -> Result<Box<dyn GpuFuture + Send + Sync>, VulkanError> {
        let mut renderer = self.renderer.lock().unwrap();
        renderer.acquire()
    }

    fn render(renderer: &Renderer) -> Box<dyn GpuFuture + Send + Sync + 'static>{
        let queue = renderer.queue.clone();
        let device = queue.device();

        let buffer = StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        );

        let mut command_buffer = AutoCommandBufferBuilder::primary(
            &buffer,
            queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit
        ).unwrap();

        let image_format = renderer.swapchain_format();

        let render_pass = single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                    format: image_format,
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

        let view = renderer.swapchain_view();
        let frame_buff = Framebuffer::new(
            render_pass,
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            }
        ).unwrap();

        command_buffer
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([22.0/255.0, 22.0/255.0, 29.0/255.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(frame_buff)
                },
                SubpassBeginInfo::default()
            ).unwrap()
            // TODO: Drawing logic
            .end_render_pass(
                SubpassEndInfo::default()
            ).unwrap();

        command_buffer.build().unwrap()
            .execute(queue.clone()).unwrap()
            .boxed_send_sync()
    }

    pub(in crate::engine) fn draw_frame(&mut self, acquired: Box<dyn GpuFuture + Send + Sync>) {
        let raw_window = Mutex::new(self.window.clone());

        let raw_renderer = self.renderer.clone();
        thread::spawn(move || {
            let window = raw_window.lock().unwrap();
            let mut renderer = raw_renderer.lock().unwrap();

            let frame = Self::render(&renderer);

            let future = frame.join(acquired).boxed();

            renderer.present(future);
            window.request_redraw();
        });
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    fn create_window(conf: &Configuration, event_loop: &EventLoop<()>) -> Result<Arc<Window>, OsError> {
        #[cfg(target_os = "windows")]
            let window_builder = {
            use winit::platform::windows::WindowBuilderExtWindows;
            WindowBuilder::new()
                .with_drag_and_drop(true)
        };

        #[cfg(not(target_os = "windows"))]
            let mut window_builder = winit::window::WindowBuilder::new();

        let window_builder = window_builder
            .with_title(conf.title.clone())
            .with_resizable(conf.resizable)
            .with_visible(false)
            .with_transparent(conf.transparent)
            .with_inner_size(conf.actual_size());

        match window_builder.build(event_loop) {
            Ok(w) => Ok(Arc::new(w)),
            Err(e) => Err(e)
        }
    }
}