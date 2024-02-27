use std::sync::Arc;
use vulkano::swapchain::{acquire_next_image, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::{single_pass_renderpass, sync, Validated, VulkanError, VulkanLibrary};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, Queue, QueueCreateInfo, QueueFlags};
use vulkano::format::{ClearValue, Format};
use vulkano::image::{Image, ImageUsage};
use vulkano::image::view::ImageView;
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, RenderPassCreateInfo};
use vulkano::sync::GpuFuture;
use winit::event_loop::EventLoop;
use winit::platform::windows::WindowBuilderExtWindows;
use winit::window::{Window, WindowBuilder};
use crate::engine::config::{Configuration, FpsLimit};

pub mod meshes;

pub struct Allocators {
    memory: Arc<StandardMemoryAllocator>,
    buffer: Arc<StandardCommandBufferAllocator>
}

impl Allocators {
    fn new(device: Arc<Device>) -> Arc<Self> {
        let memory = Arc::new(StandardMemoryAllocator::new_default(device.clone()));
        let buffer = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            StandardCommandBufferAllocatorCreateInfo::default()
        ));

        Arc::new(Self {memory, buffer})
    }
}

#[derive()]
pub struct Renderer {
    queue: Arc<Queue>,
    allocators: Arc<Allocators>,

    window: Arc<Window>,
    surface: Arc<Surface>,
    present_mode: PresentMode,
    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    image_index: u32,
    previous_frame_end: Option<Box<dyn GpuFuture + 'static>>,

    outdated: bool
}

impl Renderer {
    pub(self) fn new(present_mode: PresentMode, window: Arc<Window>, event_loop: &EventLoop<()>) -> Result<Self, Validated<VulkanError>> {
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

        let allocators = Allocators::new(device.clone());
        let previous_frame_end = Some(sync::now(device.clone()).boxed());

        Ok(Self {
                queue,
                allocators,

                window,
                present_mode,
                surface,
                swapchain,
                images,
                image_index: 0,
                previous_frame_end,

                outdated: false
            }
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

    fn recreate_swapchain_and_views(&mut self) {
        let image_extent: [u32; 2] = self.window.inner_size().into();

        if image_extent.contains(&0) {
            return;
        }

        let (new_swapchain, new_images) = self
            .swapchain
            .recreate(SwapchainCreateInfo {
                image_extent,
                present_mode: self.present_mode,
                ..self.swapchain.create_info()
            })
            .expect("failed to recreate swapchain");

        self.swapchain = new_swapchain;
        self.images = new_images;

        self.outdated = false;
    }

    pub(crate) fn acquire(&mut self) -> Result<Box<dyn GpuFuture>, VulkanError> {
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
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            self.outdated = true;
        }

        self.image_index = image_index;
        let future = self.previous_frame_end
            .take().unwrap().join(acquire_future);

        Ok(future.boxed())
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
            Ok(mut future) => {
                future.wait(None).unwrap_or_else(|e| println!("{e}"));
                self.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self.outdated = true;
                self.previous_frame_end =
                    Some(sync::now(self.queue.device().clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                self.previous_frame_end =
                    Some(sync::now(self.queue.device().clone()).boxed());
            }
        }
    }

    pub(self) fn create_window(conf: &Configuration, event_loop: &EventLoop<()>) -> Arc<Window> {
        #[cfg(target_os = "windows")]
            let mut window_builder = {
            use winit::platform::windows::WindowBuilderExtWindows;
            WindowBuilder::new()
                //.with_menu()
                .with_drag_and_drop(false)
        };

        #[cfg(not(target_os = "windows"))]
            let mut window_builder = winit::window::WindowBuilder::new();

        let window_builder = window_builder
            .with_title(conf.window_setup.title.clone())
            .with_resizable(conf.window_mode.resizable)
            .with_visible(false)
            .with_transparent(conf.window_mode.transparent)
            .with_inner_size(conf.window_mode.actual_size())
            .with_min_inner_size(conf.window_mode.min_size);

        let window = Arc::new(
            window_builder
                .build(event_loop).unwrap()
        );

        window
    }
}

pub struct GraphicsContext {
    window: Arc<Window>,
    renderer: Renderer,
    fps_limit: FpsLimit,
    current_frame: Option<Box<dyn GpuFuture + 'static>>
}

impl GraphicsContext {
    pub(super) fn new(conf: &Configuration, event_loop: &EventLoop<()>) -> Self {
        let window = Renderer::create_window(conf, event_loop);

        let present_mode = match conf.window_mode.fps_limit {
            FpsLimit::Vsync => PresentMode::FifoRelaxed,
            _ => PresentMode::Mailbox
        };

        let renderer = Renderer::new(present_mode, window.clone(), event_loop).unwrap();

        window.set_visible(conf.window_mode.visible);
        window.request_redraw();
        GraphicsContext {
            window,
            renderer,
            current_frame: None,
            fps_limit: conf.window_mode.fps_limit
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn swapchain_image(&self) -> Arc<Image> {
        self.renderer.images[self.renderer.image_index as usize].clone()
    }

    pub fn swapchain_format(&self) -> Format {
        self.renderer.swapchain.image_format()
    }

    pub fn set_fps_limit(&mut self, fps_limit: FpsLimit) {
        if self.fps_limit != fps_limit {
            self.fps_limit = fps_limit;
            self.renderer.outdated = true;
            self.renderer.present_mode = match fps_limit {
                FpsLimit::Vsync => PresentMode::FifoRelaxed,
                _ => PresentMode::Mailbox
            };
        }
    }

    pub(super) fn begin_frame(&mut self) {
        let queue = self.renderer.queue.clone();
        let device = queue.device();
        let allocators = self.renderer.allocators.clone();
        let buffer = allocators.buffer.clone();

        let clear_pass = single_pass_renderpass!(
            device.clone(),
            attachments: {
                color: {
                format: self.swapchain_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
                },
            },
            pass: { color: [color],
                depth_stencil: {},
            },
        ).unwrap();

        let image = self.swapchain_image();
        let view = ImageView::new_default(image.clone()).unwrap();
        let framebuffer = Framebuffer::new(
            clear_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            },
        ).unwrap();

        let mut builder = AutoCommandBufferBuilder::primary(
            &buffer,
            queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit
        ).unwrap();

        builder
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([0.2, 0.2, 0.2, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(framebuffer.clone())
                },
                SubpassBeginInfo::default()
            ).unwrap()
            .end_render_pass(
                SubpassEndInfo::default()
            ).unwrap();

        let future = builder.build().unwrap()
            .execute(self.renderer.queue.clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).expect("Wait error");

        self.current_frame = Some(future.boxed());
    }

    pub(super) fn end_frame(&mut self) {
        let acquire = self.renderer.acquire().unwrap();

        let future = self.current_frame
            .take().unwrap().join(acquire).boxed();

        self.renderer.present(future);
        self.window.request_redraw()
    }

    pub(super) fn resize(&mut self) {
        self.renderer.outdated = true;
    }
}