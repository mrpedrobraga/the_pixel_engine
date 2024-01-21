use engine::LuaEngine;
use gameloop::game_loop;

use render::RenderState;
use std::{fs, future::Future, sync::mpsc};
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
async fn main() {
    let e = EventLoop::new();
    let w = WindowBuilder::new()
        .with_title("Pixel Engine")
        .with_inner_size(PhysicalSize {
            width: 360,
            height: 360,
        })
        .with_visible(false)
        .with_resizable(false)
        .build(&e)
        .expect("Could not create window!");
    let s = RenderState::new(w).await;

    start(e, s);
}

enum LuaEngineMessage {
    Update(f64),
    Input(&'static str, bool),
}

enum RenderMessage {
    Clear((f64, f64, f64, f64)),
    Resize(PhysicalSize<u32>),
    ShowWindow,
}

fn start(event_loop: EventLoop<()>, render_state: RenderState) {
    let render_state_window_id = render_state.window.id();

    let (lua_sender, lua_receiver) = mpsc::channel::<LuaEngineMessage>();
    let (render_sender, render_receiver) = mpsc::channel::<RenderMessage>();

    let code = fs::read_to_string("./main.lua").expect("No main.lua file found.");

    // LUA THREAD
    let lua_render_sender = render_sender.clone();
    tokio::spawn(lua_thread(code, lua_render_sender, lua_receiver));

    // GAME LOOP
    let update_message_sender = lua_sender.clone();
    tokio::spawn(game_loop_thread(update_message_sender));

    // RENDER THREAD
    tokio::spawn(render_thread(render_receiver, render_state));

    // EVENT LOOP, on the main thread.
    let event_loop_sender = render_sender.clone();
    run_event_loop(
        event_loop,
        render_state_window_id,
        lua_sender,
        event_loop_sender,
    );
}

fn run_event_loop(
    event_loop: EventLoop<()>,
    render_state_window_id: winit::window::WindowId,
    lua_sender: mpsc::Sender<LuaEngineMessage>,
    event_loop_sender: mpsc::Sender<RenderMessage>,
) {
    event_loop.run({
        move |event, _, control_flow| match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == render_state_window_id => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(vkeycode),
                            ..
                        },
                    ..
                } => {
                    let key_is_pressed = if let ElementState::Pressed = state {
                        true
                    } else {
                        false
                    };

                    match vkeycode {
                        VirtualKeyCode::Left => {
                            let _ =
                                lua_sender.send(LuaEngineMessage::Input("left", key_is_pressed));
                        }
                        VirtualKeyCode::Right => {
                            let _ =
                                lua_sender.send(LuaEngineMessage::Input("right", key_is_pressed));
                        }
                        VirtualKeyCode::Up => {
                            let _ = lua_sender.send(LuaEngineMessage::Input("up", key_is_pressed));
                        }
                        VirtualKeyCode::Down => {
                            let _ =
                                lua_sender.send(LuaEngineMessage::Input("down", key_is_pressed));
                        }
                        VirtualKeyCode::Z => {
                            let _ = lua_sender.send(LuaEngineMessage::Input("ok", key_is_pressed));
                        }
                        VirtualKeyCode::X => {
                            let _ =
                                lua_sender.send(LuaEngineMessage::Input("cancel", key_is_pressed));
                        }
                        _ => {}
                    };
                }
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

async fn render_thread(
    render_receiver: mpsc::Receiver<RenderMessage>,
    mut render_state: RenderState,
) {
    while let Ok(message) = render_receiver.recv() {
        match message {
            RenderMessage::Clear(color) => render_state.render(color).unwrap(),
            RenderMessage::Resize(new_size) => render_state.resize(new_size),
            RenderMessage::ShowWindow => render_state.window.set_visible(true),
        }
    }
}

fn game_loop_thread(
    update_message_sender: mpsc::Sender<LuaEngineMessage>,
) -> impl Future<Output = ()> {
    game_loop(60, move |dt| {
        update_message_sender
            .send(LuaEngineMessage::Update(dt.as_secs_f64()))
            .unwrap();
    })
}

async fn lua_thread(
    code: String,
    lua_render_sender: mpsc::Sender<RenderMessage>,
    lua_receiver: mpsc::Receiver<LuaEngineMessage>,
) {
    let mut engine = LuaEngine::new(code).unwrap();
    if engine.has_draw_function() {
        lua_render_sender.send(RenderMessage::ShowWindow).unwrap()
    }

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
            LuaEngineMessage::Input(kind, pressed) => {
                let _ = engine.input(kind, pressed);
            }
        }
    }
}
