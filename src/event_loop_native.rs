#![cfg(not(target_arch = "wasm32"))]

use winit::
{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder},
};

use super::stafra_state;
use super::app_state;
use crate::app_state::RunState;

enum ResetOption
{
    Corners,
    Sides,
    Center,
    Custom
}

enum AppEvent
{
}

pub async fn run_event_loop()
{
    //Create event loop
    let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event();

    let main_window       = WindowBuilder::new().build(&event_loop).unwrap();
    let click_rule_window = WindowBuilder::new().build(&event_loop).unwrap();

    main_window.set_inner_size(winit::dpi::LogicalSize {width: 768.0, height: 768.0});
    click_rule_window.set_inner_size(winit::dpi::LogicalSize {width: 256.0, height: 256.0});

    let initial_width  = 1023;
    let initial_height = 1023;

    let mut window_size = main_window.inner_size();

    let app_state = app_state::AppState::new();

    let mut main_state = stafra_state::StafraState::new_native(&main_window, &click_rule_window, initial_width, initial_height).await;
    main_state.reset_board_standard(stafra_state::StandardResetBoardType::Corners);
    main_state.reset_click_rule();

    main_window.request_redraw();
    click_rule_window.request_redraw();

    event_loop.run(move |global_event, _, control_flow| match global_event
    {
        Event::WindowEvent {ref event, window_id} =>
        {
            let window_event = event;
            if window_id == main_window.id()
            {
                match window_event
                {
                    WindowEvent::CloseRequested =>
                    {
                        *control_flow = ControlFlow::Exit;
                    }

                    WindowEvent::Resized(physical_size) =>
                    {
                        window_size = *physical_size;
                        main_state.resize(window_size.width, window_size.height);
                    }

                    WindowEvent::ScaleFactorChanged {new_inner_size, ..} =>
                    {
                        window_size = **new_inner_size;
                        main_state.resize(window_size.width, window_size.height);
                    }

                    _ => {}
                }
            }

            if window_id == click_rule_window.id()
            {
                match window_event
                {
                    WindowEvent::Resized(physical_size) =>
                    {
                        let click_rule_size = *physical_size;
                        main_state.resize_click_rule(click_rule_size.width, click_rule_size.height);
                    }

                    WindowEvent::ScaleFactorChanged {new_inner_size, ..} =>
                    {
                        let click_rule_size = **new_inner_size;
                        main_state.resize_click_rule(click_rule_size.width, click_rule_size.height);
                    }

                    _ => {}
                }
            }
        }

        Event::RedrawRequested(_) =>
        {
            if app_state.run_state == RunState::Running
            {
                main_state.update();
            }

            match main_state.render()
            {
                Ok(_) =>
                {
                }

                Err(wgpu::SurfaceError::Lost) =>
                {
                    main_state.resize(window_size.width, window_size.height);
                }

                Err(wgpu::SurfaceError::OutOfMemory) =>
                {
                    *control_flow = ControlFlow::Exit;
                }

                Err(error) =>
                {
                    web_sys::console::log_1(&format!("{:?}", error).into());
                }
            }
        }

        Event::MainEventsCleared =>
        {
            main_window.request_redraw();
        }

        Event::UserEvent(_) =>
        {
            //Not implemented
        }

        _ => {}
    });
}