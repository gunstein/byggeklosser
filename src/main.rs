use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop}
};

mod model;
mod texture;
mod camera;
mod mouse_picker;
mod state;

fn main() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let title = env!("CARGO_PKG_NAME");
    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .build(&event_loop)
        .unwrap();

    use futures::executor::block_on;

    // Since main can't be async, we're going to need to block
    let mut appstate = block_on(state::State::new(&window));
    let mut last_render_time = std::time::Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::MainEventsCleared => window.request_redraw(),
            Event::DeviceEvent {
                ref event,
                .. // We're not using device_id currently
            } => {
                appstate.input(event);
            }
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::KeyboardInput { input, .. } => match input {
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        } => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    },
                    WindowEvent::Resized(physical_size) => {
                        appstate.resize(*physical_size);
                    },
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        appstate.resize(**new_inner_size);
                    },
                    WindowEvent::CursorMoved {position, ..}=>{
                        appstate.curr_cursor_pos = *position;
                    }, 
                    WindowEvent::MouseInput{state, button, ..}=>{
                        if *state == ElementState::Released && *button == MouseButton::Right
                        {
                            println!("state.curr_cursor_pos {:?}", appstate.curr_cursor_pos);
                            //select block under mouse
                            let coord_selected_block = mouse_picker::MousePicker::get_model_coordinates_for_voxel_under_mouse( &appstate.size, &appstate.curr_cursor_pos, &appstate.camera, &appstate.projection, &appstate.obj_model);
                        }                       
                    },
                    _ => {}
                }
            }
            // UPDATED!
            Event::RedrawRequested(_) => {
                let now = std::time::Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;
                appstate.update(dt);
                match appstate.render() {
                    Ok(_) => {}
                    // Recreate the swap_chain if lost
                    Err(wgpu::SwapChainError::Lost) => appstate.resize(appstate.size),
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SwapChainError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                    // All other errors (Outdated, Timeout) should be resolved by the next frame
                    Err(e) => eprintln!("{:?}", e),
                }
            }
            _ => {}
        }
    });
}
