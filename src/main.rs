#![allow(unused)]
use engine::LuaEngine;
use gameloop::game_loop;
use mlua::prelude::*;
use render::RenderState;
use std::{
    cell::RefCell,
    fs,
    sync::{mpsc, Arc, Mutex, RwLock},
    thread::sleep,
    time::{Duration, Instant},
};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{self, WindowBuilder},
};

pub mod engine;
pub mod gameloop;
pub mod render;

use std::io;
use std::thread::{self, JoinHandle};
use std::time;

#[tokio::main]
async fn main() -> LuaResult<()> {
    let e = EventLoop::new();
    let w = WindowBuilder::new()
        .with_title("Pixel Engine")
        .with_inner_size(PhysicalSize {
            width: 64,
            height: 64,
        })
        .with_resizable(false)
        .build(&e)
        .expect("Could not create window!");
    let s = RenderState::new(w).await;

    start(e, s)
}

enum LuaEngineMessage {
    Update(f64),
}

fn start(event_loop: EventLoop<()>, mut render_state: RenderState) -> LuaResult<()> {
    let (message_sender, message_receiver) = mpsc::channel::<LuaEngineMessage>();

    let code = fs::read_to_string("./src/test.lua")?;
    let mut render_state = Arc::new(Mutex::new(render_state));

    let mut lua_render_state = Arc::clone(&render_state);
    tokio::spawn(async move {
        let mut engine = LuaEngine::new(code).unwrap();

        engine.set_global("should_quit", false);
        engine.provide_function("clear", make_clear_function(lua_render_state));
        engine.init();

        loop {
            match message_receiver.recv().unwrap() {
                LuaEngineMessage::Update(dt) => engine.update(dt),
            }
        }
    });

    let update_message_sender = message_sender.clone();
    tokio::spawn(game_loop(60, move |dt| {
        update_message_sender.send(LuaEngineMessage::Update(dt.as_secs_f64()));
    }));

    event_loop.run({
        let render_state = Arc::clone(&render_state);
        let render_state_window_id = render_state.lock().unwrap().window.id();

        move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == render_state_window_id => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Space),
                            ..
                        },
                    ..
                } => {}
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_physical_size) => render_state
                    .lock()
                    .expect("Lock error")
                    .resize(*new_physical_size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => render_state
                    .lock()
                    .expect("Lock error")
                    .resize(**new_inner_size),

                _ => {}
            },
            _ => {}
        }
    });

    return Ok(());
}

fn handle_winit_events(event_loop: EventLoop<()>, render_state_ref: Arc<Mutex<RenderState>>) {}

fn make_clear_function<'a>(render_state_ref: Arc<Mutex<RenderState>>) -> impl IntoLua<'a> {
    mlua::Function::wrap(move |_, (color): (f64, f64, f64, f64)| {
        let mut lock = render_state_ref.lock().unwrap();
        match lock.render(color) {
            Ok(_) => {}
            Err(wgpu::SurfaceError::Lost) => lock.reconfigure_surface(),
            //Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
            Err(e) => eprintln!("{:?}", e),
        }
        drop(lock);
        Ok(())
    })
}
