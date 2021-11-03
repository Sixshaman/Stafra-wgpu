use winit::
{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

#[cfg(target_arch = "wasm32")]
use
{
    winit::platform::web::WindowBuilderExtWebSys,
    wasm_bindgen::closure::Closure
};

use wasm_bindgen::{JsCast, Clamped};
use winit::event_loop::EventLoopProxy;
use std::rc::Rc;
use super::stafra_state;

pub struct AppState
{
    event_loop:       EventLoop<AppEvent>,
    event_loop_proxy: Rc<EventLoopProxy<AppEvent>>,
    canvas_window:    Window,
    window_size:      winit::dpi::PhysicalSize<u32>,
    paused:           bool,

    #[cfg(target_arch = "wasm32")]
    document: web_sys::Document,

    #[cfg(target_arch = "wasm32")]
    save_png_function: Closure<dyn Fn()>,

    #[cfg(target_arch = "wasm32")]
    play_pause_function: Closure<dyn Fn()>,

    #[cfg(target_arch = "wasm32")]
    stop_function: Closure<dyn Fn()>,
}

enum AppEvent
{
    SavePng {},
    SwitchPlayPause {},
    Stop {}
}

impl AppState
{
    #[cfg(target_arch = "wasm32")]
    pub fn new_web() -> Self
    {
        let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event();
        let event_loop_proxy                = Rc::new(event_loop.create_proxy());

        let window   = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas   = document.get_element_by_id("STAFRA_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().ok();

        let canvas_window = WindowBuilder::new().with_canvas(canvas).build(&event_loop).unwrap();


        let event_loop_proxy_save_png = event_loop_proxy.clone();
        let save_png_function = Closure::wrap(Box::new(move ||
        {
            event_loop_proxy_save_png.send_event(AppEvent::SavePng {});
        }) as Box<dyn Fn()>);

        let save_png_button = document.get_element_by_id("button_save_png").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        save_png_button.set_onclick(Some(save_png_function.as_ref().unchecked_ref()));


        let event_loop_proxy_play_pause = event_loop_proxy.clone();
        let play_pause_function = Closure::wrap(Box::new(move ||
        {
            event_loop_proxy_play_pause.send_event(AppEvent::SwitchPlayPause {});
        }) as Box<dyn Fn()>);

        let play_pause_button = document.get_element_by_id("button_play_pause").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        play_pause_button.set_onclick(Some(play_pause_function.as_ref().unchecked_ref()));


        let event_loop_stop = event_loop_proxy.clone();
        let stop_function = Closure::wrap(Box::new(move ||
        {
            event_loop_stop.send_event(AppEvent::Stop {});
        }) as Box<dyn Fn()>);

        let stop_button = document.get_element_by_id("button_stop").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        stop_button.set_onclick(Some(stop_function.as_ref().unchecked_ref()));


        let window_size = canvas_window.inner_size();
        let paused      = false;

        Self
        {
            event_loop,
            event_loop_proxy,
            canvas_window,
            window_size,
            paused,

            document,

            save_png_function,
            play_pause_function,
            stop_function
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_native() -> Self
    {
        let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event();
        let event_loop_proxy                = Rc::new(event_loop.create_proxy());

        let canvas_window = WindowBuilder::new().build(&event_loop).unwrap();

        canvas_window.set_inner_size(winit::dpi::LogicalSize {width: 768.0, height: 768.0});
        let window_size = canvas_window.inner_size();

        let paused = false;
        Self
        {
            event_loop,
            event_loop_proxy,
            canvas_window,
            window_size,
            paused
        }
    }

    pub async fn run(self)
    {
        #[cfg(target_arch = "wasm32")]
        let document = self.document;

        let canvas_window = self.canvas_window;
        let event_loop    = self.event_loop;

        let mut window_size = self.window_size;
        let mut paused      = self.paused;

        let mut state = stafra_state::StafraState::new(&canvas_window, stafra_state::BoardDimensions {width: 1023, height: 1023}).await;
        state.reset_board();

        event_loop.run(move |event, _, control_flow| match event
        {
            Event::WindowEvent
            {
                ref event,
                window_id,
            }
            if window_id == canvas_window.id() => match event
            {
                WindowEvent::CloseRequested =>
                {
                    *control_flow = ControlFlow::Exit;
                }

                WindowEvent::Resized(physical_size) =>
                {
                    window_size = *physical_size;
                    state.resize(&window_size);
                }

                WindowEvent::ScaleFactorChanged {new_inner_size, ..} =>
                {
                    window_size = **new_inner_size;
                    state.resize(&window_size);
                }

                _ => {}
            },

            Event::RedrawRequested(_) =>
            {
                if !paused
                {
                    state.update();
                }

                match state.check_png_data_request()
                {
                    Ok((mut image_array, size, row_pitch)) =>
                    {
                        let image_data = web_sys::ImageData::new_with_u8_clamped_array(Clamped(image_array.as_mut_slice()), row_pitch).unwrap();

                        let canvas = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
                        canvas.set_width(size.width);
                        canvas.set_height(size.height);

                        let canvas_context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                        canvas_context.put_image_data(&image_data, 0.0, 0.0);

                        let link = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                        link.set_href(&canvas.to_data_url_with_type("image/png").unwrap());
                        link.set_download(&"StabilityFractal.png");
                        link.click();

                        link.remove();
                        canvas.remove();
                    },

                    Err(_) =>
                    {
                    }
                }

                match state.render()
                {
                    Ok(_) =>
                    {
                    }

                    Err(wgpu::SurfaceError::Lost) =>
                    {
                        state.resize(&window_size);
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
                canvas_window.request_redraw();
            }

            Event::UserEvent(app_event) =>
            {
                match app_event
                {
                    AppEvent::SavePng {} =>
                    {
                        #[cfg(target_arch = "wasm32")]
                        {
                            state.post_png_data_request();
                        }
                    }

                    AppEvent::SwitchPlayPause {} =>
                    {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let play_pause_button = document.get_element_by_id("button_play_pause").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
                            if paused
                            {
                                play_pause_button.set_text_content(Some("▶️"));
                            }
                            else
                            {
                                play_pause_button.set_text_content(Some("⏸️"));
                            }
                        }

                        paused = !paused;
                    }

                    AppEvent::Stop {} =>
                    {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let play_pause_button = document.get_element_by_id("button_play_pause").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
                            play_pause_button.set_text_content(Some("▶️"));
                        }

                        paused = true;
                        state.reset_board();
                    }
                }
            }

            _ => {}
        });
    }
}