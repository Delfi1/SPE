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

        #[cfg(debug_assertions)]
        update_title(ctx);
        process_event(&event, ctx, handler);

    }).expect("Event Loop error");
}

fn update_title(ctx: &Context) {
    let win_title = &ctx.conf.window_setup.title;
    let author = &ctx.conf.window_setup.author;

    let average_fps = ctx.time.average_fps().clamp(0.0, 999.0) as u64;
    let ticks = ctx.time.ticks();

    let title =
        format!(
            "{} by [{}]; Fps: {} |{}|",
            win_title,
            author,
            average_fps,
            ticks
        ).leak();

    ctx.gfx.window().set_title(title);
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
        //Todo: Vsync switch
    }

    let exit_keys = HashSet::from([1, 42]);

    if ctx.input.is_keys_pressed(exit_keys) {
        ctx.is_running = false;
    }
}

fn process_event<S: EventHandler + 'static>(event: &Event<()>, ctx: &mut Context, handler: &mut S) {
    /// Match window events
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
                ..
            } => {
                let state = event.state;
                let scancode = event.physical_key.to_scancode();

                if event.text.is_some() && !(ctx.input.is_key_pressed(29) || ctx.input.is_key_pressed(56)){
                    let text = event.text.clone().unwrap();
                    let mut chars = text.chars();

                    let input_char = chars.next().unwrap();
                    handler.char_input(input_char);
                }

                ctx.input.insert(scancode, state);
            },

            WindowEvent::DroppedFile(_buf) => {
                // When file drops
                println!("file dropped!");
                ctx.gfx.window().request_redraw();
            },

            WindowEvent::HoveredFile(_buf) => {
                println!("file hovered!");
            },

            WindowEvent::RedrawRequested => {
                // Update main states;
                ctx.time.tick();
                match_input(ctx);
                handler.update(&ctx.time, &ctx.input);
                ctx.input.update();

                // Draw frame;
                ctx.gfx.begin_frame();
                handler.draw(&mut ctx.gfx);
                ctx.gfx.end_frame();
            },

            _ => ()
        }
    }

    /// Match loop events;
    match event {
        Event::LoopExiting => {
            handler.on_quit();
        },

        _ => ()
    }
}

pub trait EventHandler {
    fn update(&mut self, _time: &TimeContext, _input: &InputContext);
    fn draw(&mut self, _gfx: &mut GraphicsContext);
    fn char_input(&mut self, _ch: char) { /* Empty */ }
    fn on_quit(&mut self) { /* Empty */ }
}