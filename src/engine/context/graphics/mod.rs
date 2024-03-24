use std::sync::Arc;
use std::time::Duration;

use vulkano::{Validated, VulkanError};
use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage};
use vulkano::buffer::allocator::{SubbufferAllocator, SubbufferAllocatorCreateInfo};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBufferAbstract, RenderPassBeginInfo, SubpassBeginInfo, SubpassEndInfo};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::descriptor_set::allocator::StandardDescriptorSetAllocator;
use vulkano::device::{DeviceExtensions, Queue};
use vulkano::image::Image;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::{GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::render_pass::{Framebuffer, Subpass};
use vulkano::swapchain::{acquire_next_image, PresentMode, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::GpuFuture;
use winit::event_loop::EventLoop;
use winit::window::Window;

use meshes::Vertex2D;

use crate::engine::context::graphics::meshes::{Mesh, Vertex3D};

use super::Config;

mod renderer;
mod meshes;

mod vs2d {
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

mod fs2d {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r#"
            #version 460

            layout(set=0, binding=0) uniform InData {
                vec2 resolution;
                float time;
            } inp;
            vec2 fragCoord = gl_FragCoord.xy;

            layout(location=0) out vec4 fragColor;

            vec3 palette(float d){
                return mix(vec3(0.2,0.7,0.9),vec3(1.,0.,1.),d);
            }

            vec2 rotate(vec2 p,float a){
                float c = cos(a);
                float s = sin(a);
                return p*mat2(c,s,-s,c);
            }

            float map(vec3 p){
                for( int i = 0; i<8; ++i){
                    float t = inp.time*0.2;
                    p.xz =rotate(p.xz,t);
                    p.xy =rotate(p.xy,t*1.89);
                    p.xz = abs(p.xz);
                    p.xz-=.5;
                }
                return dot(sign(p),p)/5.;
            }

            vec4 rm (vec3 ro, vec3 rd){
                float t = 0.;
                vec3 col = vec3(0.);
                float d;
                for(float i =0.; i<64.; i++){
                    vec3 p = ro + rd*t;
                    d = map(p)*.5;
                    if(d<0.02){
                        break;
                    }
                    if(d>100.){
                        break;
                    }
                    //col+=vec3(0.6,0.8,0.8)/(400.*(d));
                    col+=palette(length(p)*.1)/(400.*(d));
                    t+=d;
                }
                return vec4(col,1./(d*100.));
            }
            void main()
            {
                vec2 uv = (fragCoord-(inp.resolution.xy/2.))/inp.resolution.x;
                vec3 ro = vec3(0.,0.,-50.);
                ro.xz = rotate(ro.xz,inp.time);
                vec3 cf = normalize(-ro);
                vec3 cs = normalize(cross(cf,vec3(0.,1.,0.)));
                vec3 cu = normalize(cross(cf,cs));

                vec3 uuv = ro+cf*3. + uv.x*cs + uv.y*cu;

                vec3 rd = normalize(uuv-ro);

                vec4 col = rm(ro,rd);

                fragColor = col;
            }
        "#,
    }
}

mod vs3d {
    vulkano_shaders::shader! {
        ty: "vertex",
        src: r"
            #version 460

            layout(location = 0) in vec3 position;

            void main() {
                gl_Position = vec4(position, 1.0);
            }
        ",
    }
}

mod fs3d {
    vulkano_shaders::shader! {
        ty: "fragment",
        src: r"
            #version 460

            void main() {

            }
        ",
    }
}

/// Graphics context;
/// Window, surface,
/// Swapchain, draw_data, image;
pub struct GraphicsContext {
    queue: Arc<Queue>,

    window: Arc<Window>,
    surface: Arc<Surface>,

    swapchain: Arc<Swapchain>,
    images: Vec<Arc<Image>>,
    image_index: usize,

    pipeline3d: Arc<GraphicsPipeline>,
    pipeline2d: Arc<GraphicsPipeline>,

    memory_alloc: Arc<StandardMemoryAllocator>,
    buffer_alloc: Arc<StandardCommandBufferAllocator>,
    descriptor_alloc: Arc<StandardDescriptorSetAllocator>,

    vsync: bool,
    outdated: bool,
    resized: bool,
}

fn create_pipelines(queue: Arc<Queue>, frame_buffer: Arc<Framebuffer>, window: Arc<Window>) -> (Arc<GraphicsPipeline>, Arc<GraphicsPipeline>) {
    let device = queue.device();

    let render_pass = frame_buffer.render_pass();
    let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

    let size: [u32; 2] = window.inner_size().into();

    let viewport = match size.contains(&0) {
        true => Viewport::default(),
        false => Viewport {
            offset: [0.0, 0.0],
            extent: [size[0] as f32, size[1] as f32],
            depth_range: 0.0..=1.0,
        }
    };

    let pipeline3d = {
        let vs = vs3d::load(device.clone()).unwrap();
        let fs = fs3d::load(device.clone()).unwrap();

        let vertex_shader = vs.entry_point("main").unwrap();
        let fragment_shader = fs.entry_point("main").unwrap();

        let vertex_input_state = Vertex3D::per_vertex()
            .definition(&vertex_shader.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vertex_shader),
            PipelineShaderStageCreateInfo::new(fragment_shader),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        ).unwrap();

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.clone().into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState {
                    viewports: [viewport.clone()].into_iter().collect(),
                    ..Default::default()
                }),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState::default(),
                )),
                subpass: Some(subpass.clone().into()),
                ..GraphicsPipelineCreateInfo::layout(layout.clone())
            },
        ).unwrap()
    };

    let pipeline2d = {
        let vs = vs2d::load(device.clone()).unwrap();
        let fs = fs2d::load(device.clone()).unwrap();

        let vertex_shader = vs.entry_point("main").unwrap();
        let fragment_shader = fs.entry_point("main").unwrap();

        let vertex_input_state = Vertex2D::per_vertex()
            .definition(&vertex_shader.info().input_interface)
            .unwrap();

        let stages = [
            PipelineShaderStageCreateInfo::new(vertex_shader),
            PipelineShaderStageCreateInfo::new(fragment_shader),
        ];

        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        ).unwrap();

        GraphicsPipeline::new(
            device.clone(),
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

    (pipeline3d, pipeline2d)
}

impl GraphicsContext {
    pub(super) fn new(config: &Config, event_loop: &EventLoop<()>) -> Self {
        let instance = renderer::init_vulkan(event_loop);

        let extensions = DeviceExtensions {
            khr_swapchain: true,
            ..DeviceExtensions::empty()
        };

        let window = renderer::init_window(event_loop, config);

        let surface = Surface::from_window(instance.clone(), window.clone())
            .expect("Surface create error");

        let (phys_device, queue_index) =
            renderer::fetch_phys_device(instance.clone(), &surface, &extensions);

        let queue = renderer::create_graphics_queue(phys_device.clone(), queue_index, &extensions);
        let device = queue.device();

        let (swapchain, images) =
            renderer::create_swapchain(phys_device.clone(), &surface, &window, device.clone());

        let memory_alloc = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

        let buffer_alloc = Arc::new(StandardCommandBufferAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let descriptor_alloc = Arc::new(StandardDescriptorSetAllocator::new(
            device.clone(),
            Default::default(),
        ));

        let image = images[0].clone();
        let frame_buffer =
            renderer::create_frame_buffer(queue.clone(), swapchain.clone(), image);

        let (pipeline3d, pipeline2d) =
            create_pipelines(queue.clone(), frame_buffer.clone(), window.clone());

        Self {
            queue,

            window,
            surface,

            swapchain,
            images,
            image_index: 0,

            pipeline3d,
            pipeline2d,
            memory_alloc,
            buffer_alloc,
            descriptor_alloc,

            vsync: true,
            outdated: true,
            resized: true,
        }
    }

    pub(in crate::engine) fn resized(&mut self) {
        self.resized = true;
    }

    pub(in crate::engine) fn acquire(&mut self) -> Result<Box<dyn GpuFuture + Send + Sync>, VulkanError> {
        if self.outdated | self.resized {
            let present_mode = match self.vsync {
                true => PresentMode::FifoRelaxed,
                false => PresentMode::Mailbox
            };

            let size: [u32; 2] = self.window.inner_size().into();
            let image_extent = match size.contains(&0) {
                true => [1, 1],
                false => size
            };

            // Recreate swapchain
            let (new_swapchain, new_images) = self.swapchain.recreate(
                SwapchainCreateInfo {
                    image_extent,
                    present_mode,
                    ..self.swapchain.create_info()
                }
            ).unwrap();

            let image= new_images[self.image_index].clone();
            let frame_buffer =
                renderer::create_frame_buffer(self.queue.clone(), new_swapchain.clone(), image);

            if self.resized {
                let (new_3d, new_2d) =
                    create_pipelines(self.queue.clone(), frame_buffer, self.window.clone());

                self.pipeline3d = new_3d;
                self.pipeline2d = new_2d;

                self.resized = false;
            }

            self.swapchain = new_swapchain;
            self.images = new_images;

            self.outdated = false;
        };

        let (image_index, suboptimal, acquire_future) =
            match acquire_next_image(self.swapchain.clone(), None)
                .map_err(Validated::unwrap)
            {
                Ok(r) => r,
                Err(VulkanError::OutOfDate) => {
                    self.outdated = true;
                    return Err(VulkanError::OutOfDate);
                }
                Err(VulkanError::NotReady) => {
                    return Err(VulkanError::NotReady);
                }
                Err(_e) => panic!("failed to acquire next image: {_e}"),
            };

        if suboptimal {
            self.outdated = true;
        }

        self.image_index = image_index as usize;

        Ok(acquire_future.boxed_send_sync())
    }

    pub fn window(&self) -> &Arc<Window> {
        &self.window
    }

    pub fn set_vsync(&mut self, vsync: bool) {
        if self.vsync != vsync {
            self.outdated = true;
            self.vsync = vsync;
        }
    }

    pub fn is_vsync(&self) -> bool {
        self.vsync
    }

    pub fn redraw(&mut self, acquired: Box<dyn GpuFuture + Send + Sync>, init_time: Duration) {
        let mut command_buffer = AutoCommandBufferBuilder::primary(
            &self.buffer_alloc,
            self.queue.queue_family_index(),
            CommandBufferUsage::MultipleSubmit,
        ).unwrap();

        let image = self.images[self.image_index].clone();
        let frame_buffer = renderer::create_frame_buffer(self.queue.clone(), self.swapchain.clone(), image);

        let uniform_buffer = SubbufferAllocator::new(
            self.memory_alloc.clone(),
            SubbufferAllocatorCreateInfo {
                buffer_usage: BufferUsage::UNIFORM_BUFFER,
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
        );

        let resolution: [f32; 2] = self.window.inner_size().into();

        let subbuffer = {
            let uniform_data = fs2d::InData {
                resolution,
                time: init_time.as_secs_f32(),
            };

            let subbuffer = uniform_buffer.allocate_sized().unwrap();
            *subbuffer.write().unwrap() = uniform_data;

            subbuffer
        };

        let layout = self.pipeline2d.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            &self.descriptor_alloc,
            layout.clone(),
            [WriteDescriptorSet::buffer(0, subbuffer)],
            [],
        ).unwrap();

        let vertex1 = Vertex2D { position: [-1.0, -1.0] };
        let vertex2 = Vertex2D { position: [1.0, -1.0] };
        let vertex3 = Vertex2D { position: [1.0, 1.0] };
        let vertex4 = Vertex2D { position: [1.0, 1.0] };
        let vertex5 = Vertex2D { position: [-1.0, 1.0] };
        let vertex6 = Vertex2D { position: [-1.0, -1.0] };

        let vertex_buffer = Buffer::from_iter(
            self.memory_alloc.clone(),
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

        command_buffer
            .begin_render_pass(
                RenderPassBeginInfo {
                    clear_values: vec![Some([22.0 / 255.0, 22.0 / 255.0, 29.0 / 255.0, 1.0].into())],
                    ..RenderPassBeginInfo::framebuffer(frame_buffer)
                },
                SubpassBeginInfo::default(),
            ).unwrap()
            .bind_pipeline_graphics(self.pipeline2d.clone())
            .unwrap()
            .bind_descriptor_sets(
                PipelineBindPoint::Graphics,
                self.pipeline2d.layout().clone(),
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
                .execute(self.queue.clone()).unwrap()
                .join(acquired).boxed();

        let future = after_future
            .then_swapchain_present(
                self.queue.clone(),
                SwapchainPresentInfo::swapchain_image_index(
                    self.swapchain.clone(),
                    self.image_index as u32,
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

        self.window.request_redraw();
    }
}