use engine::LuaEngine;
use gameloop::game_loop;
use mlua::prelude::*;
use render::RenderState;
use std::{fs, sync::mpsc};
use winit::{
    dpi::PhysicalSize,
    event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

pub mod engine;
pub mod gameloop;
pub mod render;

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

enum RenderMessage {
    Clear((f64, f64, f64, f64)),
    Resize(PhysicalSize<u32>),
}

fn start(event_loop: EventLoop<()>, mut render_state: RenderState) -> LuaResult<()> {
    let render_state_window_id = render_state.window.id();

    let (lua_sender, lua_receiver) = mpsc::channel::<LuaEngineMessage>();
    let (render_sender, render_receiver) = mpsc::channel::<RenderMessage>();

    let code = fs::read_to_string("./src/test.lua")?;

    let lua_render_sender = render_sender.clone();
    tokio::spawn(async move {
        let mut engine = LuaEngine::new(code).unwrap();

        engine
            .set_global("should_quit", false)
            .expect("Error setting should_quit global!");
        engine
            .provide_function(
                "clear",
                mlua::Function::wrap(move |_, color: (f64, f64, f64, f64)| {
                    lua_render_sender.send(RenderMessage::Clear(color)).unwrap();
                    Ok(())
                }),
            )
            .expect("Error defining clear function!");
        engine.init().unwrap();

        while let Ok(message) = lua_receiver.recv() {
            match message {
                LuaEngineMessage::Update(dt) => {
                    engine.update(dt).unwrap();
                    engine.draw().unwrap();
                }
            }
        }
    });

    let update_message_sender = lua_sender.clone();
    tokio::spawn(game_loop(60, move |dt| {
        update_message_sender
            .send(LuaEngineMessage::Update(dt.as_secs_f64()))
            .unwrap();
    }));

    tokio::spawn(async move {
        while let Ok(message) = render_receiver.recv() {
            match message {
                RenderMessage::Clear(color) => render_state.render(color).unwrap(),
                _ => {}
            }
        }
    });

    let event_loop_sender = render_sender.clone();
    event_loop.run({
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
                WindowEvent::Resized(new_physical_size) => event_loop_sender
                    .send(RenderMessage::Resize(*new_physical_size))
                    .unwrap(),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => event_loop_sender
                    .send(RenderMessage::Resize(**new_inner_size))
                    .unwrap(),

                _ => {}
            },
            _ => {}
        }
    });
}
