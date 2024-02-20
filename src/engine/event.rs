use std::collections::HashSet;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::scancode::PhysicalKeyExtScancode;
use winit::window::Fullscreen;
use crate::engine::graphics::GraphicsContext;
use crate::engine::input::InputContext;
use crate::engine::time::TimeContext;
use super::context::*;

pub fn run<S: EventHandler + 'static>(event_loop: EventLoop<()>, mut context: Context, mut app: S) {
    event_loop.set_control_flow(ControlFlow::Poll);

    event_loop.run( move |event, target| {
        let ctx = &mut context;
        let handler = &mut app;

        if !ctx.is_running {
            target.exit()
        }

        //#[cfg(debug_assertions)]
        {
            let win_title = &ctx.conf.window_setup.title;
            let author = &ctx.conf.window_setup.author;

            let average_fps = ctx.time.average_fps().clamp(0.0, 999.0) as u64;
            let title =
                format!(
                    "{} by [{}]; Fps: {} ",
                    win_title,
                    author,
                    average_fps
                ).leak();

            ctx.gfx.window.set_title(title);
        }

        process_event(&event, ctx, handler);
        match event {
            Event::LoopExiting => {
                handler.on_quit();
            },

            Event::AboutToWait => {
                ctx.time.tick();

                match_input(ctx);
                handler.update(&ctx.time, &ctx.input);

                ctx.gfx.begin_frame();
                handler.draw(&mut ctx.gfx);
                ctx.gfx.end_frame();

                ctx.input.update();
            }

            _ => ()
        }

    }).expect("Event Loop error");
}

fn match_input(ctx: &mut Context) {
    if ctx.input.is_key_just_pressed(87) {
        let window = ctx.gfx.window();

        let is_fullscreen = window.fullscreen().is_some();
        window.set_fullscreen(if !is_fullscreen {
            Some(Fullscreen::Borderless(window.current_monitor()))
        } else {
            None
        });
    }

    if ctx.input.is_key_pressed(29) && ctx.input.is_key_just_pressed(50) {
        let window = ctx.gfx.window();

        window.set_maximized(!window.is_maximized())
    }

    if ctx.input.is_key_pressed(29) && ctx.input.is_key_just_pressed(47) {
        ctx.gfx.set_vsync(!ctx.gfx.is_vsync())
    }

    let exit_keys = HashSet::from([1, 42]);

    if ctx.input.is_keys_pressed(exit_keys) {
        ctx.is_running = false;
    }
}

fn process_event<S: EventHandler + 'static>(event: &Event<()>, ctx: &mut Context, handler: &mut S) {
    if let Event::WindowEvent { event, .. } = event {
        match event {
            WindowEvent::CloseRequested => {
                ctx.is_running = false;
            },

            WindowEvent::Resized(_) => {
                ctx.gfx.resize()
            }

            WindowEvent::KeyboardInput {
                event,
                is_synthetic,
                ..
            } => {
                let state = event.state;
                let text = event.text.clone();
                let scancode = event.physical_key.to_scancode();

                if text.is_some() {
                    let byte = text.clone()
                        .unwrap().as_bytes()
                        .first().unwrap()
                        .clone();

                    let input_char = char::from(byte);
                    handler.char_input(input_char);
                }

                ctx.input.insert(scancode, state, text);
            },

            _ => ()
        }
    }
}

pub trait EventHandler {
    fn update(&mut self, _time: &TimeContext, _input: &InputContext);
    fn draw(&mut self, _gfx: &mut GraphicsContext);

    fn on_quit(&mut self) { /* Empty */ }

    fn char_input(&mut self, ch: char) { /* Empty */ }
}