use std::sync::Arc;
use std::thread;
use std::time::Duration;
use vulkano::device::physical::PhysicalDevice;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, DeviceOwned, Queue, QueueCreateInfo, QueueFlags};
use vulkano::image::{Image, ImageCreateInfo, ImageType, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::swapchain::{acquire_next_image, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use vulkano::{single_pass_renderpass, sync, Validated, VulkanError, VulkanLibrary};
use vulkano::command_buffer::{AutoCommandBufferBuilder, ClearColorImageInfo, CommandBufferUsage, PrimaryCommandBufferAbstract, RenderPassBeginInfo, SecondaryCommandBufferAbstract, SubpassBeginInfo, SubpassContents, SubpassEndInfo};
use vulkano::command_buffer::allocator::{StandardCommandBufferAllocator, StandardCommandBufferAllocatorCreateInfo};
use vulkano::format::{ClearColorValue, ClearValue, Format};
use vulkano::image::view::ImageView;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass};
use winit::event_loop::EventLoop;
use winit::window::{CursorGrabMode, Fullscreen, Window, WindowBuilder};
use crate::engine::config::{Configuration, FpsLimit, WindowMode};
use crate::engine::time::TimeContext;

pub struct GraphicsContext {
    pub(crate) window: Arc<Window>,
    pub(crate) surface: Arc<Surface>,

    pub(crate) memory_alloc: Arc<StandardMemoryAllocator>,
    pub(crate) buff_alloc: Arc<StandardCommandBufferAllocator>,
    pub(crate) swapchain: Arc<Swapchain>,
    pub(crate) present_mode: PresentMode,
    pub(crate) fps_limit: FpsLimit,
    pub(crate) main_view: Arc<ImageView>,
    pub(crate) images: Vec<Arc<Image>>,

    pub(crate) previous_frame_end: Option<Box<dyn GpuFuture + 'static>>,
    pub(crate) current_frame: Option<Box<dyn GpuFuture + Send + Sync + 'static>>,
    pub(crate) render_pass: Option<Arc<RenderPass>>,

    pub(crate) image_index: u32,

    pub(crate) graphics_queue: Arc<Queue>,
    pub(crate) compute_queue: Arc<Queue>,

    pub(crate) recreate_swapchain: bool
}

fn get_queue_with_flag(physical_device: Arc<PhysicalDevice>, flags: QueueFlags, extensions: DeviceExtensions) -> Arc<Queue> {
    let queue_family_index = physical_device
        .queue_family_properties()
        .iter()
        .enumerate()
        .position(|(_queue_family_index, queue_family_properties)| {
            queue_family_properties.queue_flags.contains(flags)
        })
        .expect("couldn't find a graphical queue family") as u32;

    let (_, mut queues) = Device::new(
        physical_device,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: extensions,
            ..Default::default()
        },
    ).expect("failed to create device");

    queues.next().unwrap()
}

impl GraphicsContext {
    pub(crate) fn new(conf: &Configuration, event_loop: &EventLoop<()>) -> Self {
        let library = VulkanLibrary::new()
            .expect("no local Vulkan library/DLL");

        let required_extensions = Surface::required_extensions(&event_loop);
        let instance = Instance::new(
            library,
            InstanceCreateInfo {
                enabled_extensions: required_extensions,
                ..InstanceCreateInfo::default()
            }
        ).expect("failed to create instance");

        let physical_device = instance
            .enumerate_physical_devices()
            .expect("could not enumerate devices")
            .next()
            .expect("no devices available");

        let graphics_queue = get_queue_with_flag(
            physical_device.clone(),
            QueueFlags::GRAPHICS,
            DeviceExtensions {
                khr_swapchain: true,
                ..DeviceExtensions::default()
            }
        );

        let compute_queue = get_queue_with_flag(
            physical_device.clone(),
            QueueFlags::COMPUTE,
            DeviceExtensions::default()
        );

        let window = Self::create_window(conf, event_loop);

        let surface = Surface::from_window(
            instance.clone(),
            window.clone()
        ).expect("Surface creation error");

        let caps = physical_device
            .surface_capabilities(&surface, Default::default())
            .expect("failed to get surface capabilities");

        let composite_alpha = caps
            .supported_composite_alpha
            .into_iter()
            .next()
            .unwrap();

        let image_format = physical_device
            .surface_formats(&surface, Default::default())
            .expect("Format not found")[0].0;

        let present_mode = match conf.window_mode.fps_limit {
            FpsLimit::Vsync => PresentMode::FifoRelaxed,
            _ => PresentMode::Mailbox
        };

        let (swapchain, images) = Swapchain::new(
            graphics_queue.device().clone(),
            surface.clone(),
            SwapchainCreateInfo {
                min_image_count: caps.min_image_count + 1,
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                // Fps mode;
                present_mode,
                composite_alpha,

                ..SwapchainCreateInfo::default()
            }
        ).expect("Swapchain creation error");

        let previous_frame_end = Some(sync::now(graphics_queue.device().clone()).boxed());

        let buff_alloc =
            Arc::new(
                StandardCommandBufferAllocator::new(
                    graphics_queue.device().clone(),
                    StandardCommandBufferAllocatorCreateInfo::default()
                )
            );

        let memory_alloc =
            Arc::new(
                StandardMemoryAllocator::new_default(
                    graphics_queue.device().clone()
                )
            );

        let image = images.first().unwrap();
        let main_view = ImageView::new_default(
            Image::new(
                memory_alloc.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format: Format::R8G8B8A8_UNORM,
                    extent: image.extent(),
                    usage: ImageUsage::SAMPLED | ImageUsage::STORAGE | ImageUsage::TRANSFER_DST,
                    ..Default::default()
                },
                AllocationCreateInfo::default(),
            ).unwrap(),
        ).unwrap();

        let fps_limit = conf.window_mode.fps_limit.clone();

        Self {
            window,
            surface,
            present_mode,
            fps_limit,
            buff_alloc,
            memory_alloc,
            main_view,

            images,
            image_index: 0,
            previous_frame_end,
            current_frame: None,
            render_pass: None,

            swapchain,
            graphics_queue,
            compute_queue,
            recreate_swapchain: false
        }
    }

    pub fn is_vsync(&self) -> bool {
        match self.present_mode {
            PresentMode::FifoRelaxed => true,
            _ => false
        }
    }

    pub fn set_fps(&mut self, fps_limit: FpsLimit) {
        let mode = match fps_limit {
            FpsLimit::Vsync => PresentMode::FifoRelaxed,
            _ => PresentMode::Mailbox
        };

        if self.fps_limit != fps_limit {
            self.fps_limit = fps_limit;
            self.present_mode = mode;
            self.recreate_swapchain = true;
        }
    }

    pub fn create_window(conf: &Configuration, event_loop: &EventLoop<()>) -> Arc<Window> {
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

    pub fn surface(&self) -> Arc<Surface> {
        self.surface.clone()
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub(crate) fn update_main_view(&mut self) {
        let image = self.images.first().unwrap();

        let format = self.main_view.format();
        let usage = self.main_view.usage();

        self.main_view = ImageView::new_default(
            Image::new(
                self.memory_alloc.clone(),
                ImageCreateInfo {
                    image_type: ImageType::Dim2d,
                    format,
                    extent: image.extent(),
                    usage,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE,
                    ..Default::default()
                },
            ).unwrap(),
        ).unwrap();
    }

    pub(crate) fn main_view(&self) -> Arc<ImageView> {
        self.main_view.clone()
    }

    pub(crate) fn grab(&self, grabbed: bool) {
        let window = self.window();

        let mode = match grabbed {
            true => CursorGrabMode::Confined,
            false => CursorGrabMode::None
        };

        match window.set_cursor_grab(mode) {
            Ok(_) => {}
            Err(winit::error::ExternalError::NotSupported(_)) => {}
            Err(err) => panic!("{:?}", err),
        }
    }

    pub(crate) fn recreate_swapchain_and_views(&mut self) {
        let image_extent: [u32; 2] = self.window().inner_size().into();

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

        self.update_main_view();
        self.recreate_swapchain = false;
    }

    pub(crate) fn swapchain_view(&self) -> Arc<ImageView> {
        let image = self.images[self.image_index as usize].clone();

        ImageView::new_default(image).unwrap()
    }

    #[inline]
    pub(crate) fn acquire(&mut self) -> Result<Box<dyn GpuFuture>, VulkanError> {
        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return Err(VulkanError::OutOfDate);
                }
                Err(e) => panic!("failed to acquire next image: {e}"),
            };

        if suboptimal {
            self.recreate_swapchain = true;
        }

        self.image_index = image_index;

        let future = self.previous_frame_end.take().unwrap().join(acquire_future);

        Ok(future.boxed())
    }

    pub fn get_render_pass(&mut self) -> Arc<RenderPass> {
        self.render_pass.take().unwrap().clone()
    }

    /// Clear current color image;
    pub(crate) fn begin_frame(&mut self) {
        let buff_alloc = self.buff_alloc.clone();
        let queue = self.graphics_queue.clone();

        let mut builder = AutoCommandBufferBuilder::primary(
            &buff_alloc.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        ).unwrap();

        let clear_pass = single_pass_renderpass!(
            self.graphics_queue.device().clone(),
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
            }
        ).unwrap();

        let view = self.swapchain_view();
        let framebuffer = Framebuffer::new(
            clear_pass.clone(),
            FramebufferCreateInfo {
                attachments: vec![view],
                ..Default::default()
            }
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
            .execute(self.graphics_queue.clone())
            .unwrap()
            .then_signal_fence_and_flush()
            .unwrap();

        future.wait(None).expect("Wait error");
        self.render_pass = Some(clear_pass);
        self.current_frame = Some(future.boxed_send_sync());

    }

    pub(crate) fn end_frame(&mut self, time_context: &TimeContext) {
        if self.current_frame.is_none() {
            panic!("Frame is none");
        }

        if self.recreate_swapchain {
            self.recreate_swapchain_and_views();
        }

        let acquire = match self.acquire() {
            Ok(_ac) => _ac,
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                return;
            }
            Err(_e) => panic!("Acquire error")
        };

        let after_future = self.current_frame
            .take().unwrap().join(acquire).boxed();

        self.present(after_future, time_context);
        //println!("End frame");
    }

    #[inline]
    pub fn present(&mut self, after_future: Box<dyn GpuFuture>, time_context: &TimeContext) {
        let future = after_future
            .then_swapchain_present(
                self.graphics_queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    self.image_index,
                ),
            )
            .then_signal_fence_and_flush();

        match future.map_err(Validated::unwrap) {
            Ok(mut future) => {
                match self.fps_limit {
                    FpsLimit::Vsync | FpsLimit::Inf => {
                        future.wait(None).unwrap_or_else(|e| println!("{e}"));
                    },
                    FpsLimit::Set(value) => {
                        let frame_time = Duration::from_secs_f64(1.0 / value as f64);

                        future.wait(None).unwrap_or_else(|e| println!("{e}"));
                        let delta = time_context.delta();

                        println!("Delta: {}", delta.as_secs_f64());
                        println!("{}", frame_time > delta);
                        match frame_time > delta {
                            true => thread::sleep(frame_time - delta),
                            false => ()
                        };
                    }
                }
                self.previous_frame_end = Some(future.boxed());
            }
            Err(VulkanError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end =
                    Some(sync::now(self.graphics_queue.device().clone()).boxed());
            }
            Err(e) => {
                println!("failed to flush future: {e}");
                self.previous_frame_end =
                    Some(sync::now(self.graphics_queue.device().clone()).boxed());
            }
        }

    }

    pub(crate) fn resize(&mut self) {
        self.recreate_swapchain = true;
    }
}