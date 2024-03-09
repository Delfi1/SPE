use vulkano::VulkanError;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::platform::scancode::PhysicalKeyExtScancode;

use super::context::Context;

/// Event worker - main Engine struct;
/// It contains main app loop and app callbacks;
pub struct EventWorker<Handler>
where Handler: EventHandler + Send + Sync + Sized + 'static {
    app: Handler,
    context: Context
}

impl<Handler> EventWorker<Handler>
where Handler: EventHandler + Send + Sync + Sized + 'static {
    pub fn new(mut context: Context) -> Self {
        let mut app = Handler::create(&mut context);

        Self {
            app,
            context
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
        let mut context = self.context;
        let mut app = self.app;

        event_loop.set_control_flow(ControlFlow::Poll);

        event_loop.run(move |event, target| {
            let ctx = &mut context;
            let handler = &mut app;

            update_title(ctx);

            match event {
                Event::WindowEvent {
                    event,
                    ..
                } => {
                    match event {
                        WindowEvent::CloseRequested => {
                            target.exit();
                        },
                        WindowEvent::Resized(_) | WindowEvent::ScaleFactorChanged {..} => {
                            ctx.graphics.set_outdated();
                        },
                        WindowEvent::RedrawRequested => {
                            let acquired = match ctx.graphics.acquire() {
                                Ok(ac) => ac,
                                Err(VulkanError::OutOfDate) => {
                                    // If acquire is out of date -> continue && redraw;
                                    ctx.graphics.set_outdated();
                                    ctx.graphics.window().request_redraw();
                                    return;
                                }
                                Err(_e) => panic!("Acquire error")
                            };

                            ctx.time.tick();
                            ctx.graphics.draw_frame(acquired);
                        },
                        WindowEvent::KeyboardInput {
                            event,
                            ..
                        } => {
                            let code = event.physical_key.to_scancode();
                            let state = event.state;

                            if code.is_some() {
                                let code = code.unwrap();
                                ctx.input.insert(code, state);
                            }
                        },
                        _ => ()
                    }
                }
                Event::LoopExiting => {
                    handler.on_quit();
                }
                _ => ()
            }

            match_input(ctx, target);
            ctx.input.update();
        }).expect("Event loop error");
    }
}

fn match_input(ctx: &mut Context, target: &EventLoopWindowTarget<()>) {
    if ctx.input.is_key_pressed(29) && ctx.input.is_key_just_pressed(50) {
        let window = ctx.graphics.window();
        window.set_maximized(!window.is_maximized());
    }

    if ctx.input.is_keys_pressed(&[1, 42]) {
        target.exit();
    }
}

fn update_title(ctx: &Context) {
    let win_title = &ctx.conf.title;
    let author = &ctx.conf.author;

    let average_fps = ctx.time.average_fps().clamp(0.0, 999.0) as u64;

    let title =
        format!(
            "{} by [{}]; Fps: {}",
            win_title,
            author,
            average_fps
        ).leak();

    ctx.graphics.window().set_title(title);
}

pub trait EventHandler: Sized + Send + Sync {
    /// Create engine application function;
    fn create(_ctx: &Context) -> Self;

    /// Update function what provides update of engine state
    /// DeltaTime + Input data;
    fn update(&mut self) { /* Empty */ }

    /// Draw function helps user with Mesh's drawing;
    fn draw(&self) { /* Empty */ }

    /// On quit engine callback;
    fn on_quit(&mut self) { /* Empty */ }
}