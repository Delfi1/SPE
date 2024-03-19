use vulkano::VulkanError;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::keyboard::KeyCode;
use winit::platform::scancode::PhysicalKeyExtScancode;
use winit::window::Fullscreen;

use context::Context;

pub mod context;

/// Worker helps with context x application logic;
pub struct Worker<Handler>
    where Handler: EventHandler
{
    context: Context,
    handler: Handler,
}

impl<Handler> Worker<Handler>
    where Handler: EventHandler
{
    /// Create application worker;
    pub fn new(mut context: Context) -> Self {
        Self {
            handler: Handler::setup(&mut context),
            context,
        }
    }

    /// Run application main loop;
    pub fn run(self, event_loop: EventLoop<()>) {
        event_loop.set_control_flow(ControlFlow::Poll);

        let mut context = self.context;
        let mut app = self.handler;

        event_loop.run(move |event, target| {
            let ctx = &mut context;
            let handler = &mut app;
            Self::match_event(event, ctx, handler, target);
        }).unwrap();
    }

    fn match_input(ctx: &mut Context, target: &EventLoopWindowTarget<()>) {
        if ctx.input.is_key_pressed(KeyCode::ControlLeft) && ctx.input.is_key_just_pressed(KeyCode::KeyM) {
            let window = ctx.graphics.window();
            window.set_maximized(!window.is_maximized());
        }

        if ctx.input.is_key_just_pressed(KeyCode::F11) {
            let window = ctx.graphics.window();
            let is_fullscreen = window.fullscreen().is_some();
            match is_fullscreen {
                true => window.set_fullscreen(None),
                false => window.set_fullscreen(Some(Fullscreen::Borderless(window.current_monitor())))
            };
        }

        if ctx.input.is_keys_pressed(&[KeyCode::Escape, KeyCode::ShiftLeft]) {
            target.exit();
        }

        ctx.input.update();
    }

    /// Update window title;
    fn update_title(ctx: &Context) {
        let win_title = &ctx.title;
        let author = &ctx.author;

        let average_fps = ctx.time.average_fps();

        let title =
            format!(
                "{} by [{}]; Fps: {}",
                win_title,
                author,
                average_fps
            ).leak();

        ctx.graphics.window().set_title(title);
    }

    /// Match user keyboard input || window events;
    fn match_event(event: Event<()>, ctx: &mut Context, handler: &mut Handler, target: &EventLoopWindowTarget<()>) {
        match event {
            Event::NewEvents(StartCause::Poll) => {
                Self::match_input(ctx, target);
                Self::update_title(ctx);
            }
            Event::WindowEvent {
                event,
                ..
            } => {
                match event {
                    WindowEvent::CloseRequested => {
                        target.exit();
                    },
                    WindowEvent::Resized(..) | WindowEvent::ScaleFactorChanged { .. } => {
                        ctx.graphics.resized()
                    }

                    WindowEvent::KeyboardInput {
                        event,
                        ..
                    } => {
                        ctx.input.insert(event);
                    }
                    WindowEvent::CursorMoved {
                        position,
                        ..
                    } => {
                        let pos: [f64; 2] = position.into();
                        //ctx.graphics.draw();
                        println!("{:?}", pos)
                    },
                    WindowEvent::RedrawRequested => {
                        let acquired = match ctx.graphics.acquire() {
                            Ok(ac) => ac,
                            Err(VulkanError::OutOfDate) => {
                                // If acquire is out of date -> continue && redraw;
                                ctx.graphics.window().request_redraw();
                                return;
                            }
                            Err(_e) => panic!("Acquire error")
                        };

                        ctx.time.tick();

                        handler.on_update();
                        handler.on_draw();

                        ctx.graphics.redraw(acquired);
                    }
                    _ => ()
                }
            }
            Event::LoopExiting => {
                handler.on_quit()
            }
            _ => ()
        }
    }
}

pub trait EventHandler: Sized + Send + Sync {
    fn setup(_context: &mut Context) -> Self;

    fn on_update(&mut self) { /* Empty */ }

    fn on_draw(&self) { /* Empty */ }

    fn on_quit(&self) { /* Empty */ }
}