use winit::
{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, Window},
};

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowBuilderExtWebSys;

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

    #[cfg(target_arch = "wasm32")]
    document: web_sys::Document,

    #[cfg(target_arch = "wasm32")]
    save_png_function: Closure<dyn Fn()>,
}

enum AppEvent
{
    SavePng,
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
        let canvas   = document.unwrap().get_element_by_id("STAFRA_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().ok();

        let canvas_window = WindowBuilder::new().with_canvas(canvas).build(&event_loop).unwrap();

        let event_loop_proxy_cloned = event_loop_proxy.clone();
        let save_png_function = Closure::wrap(Box::new(move ||
        {
            event_loop_proxy_cloned.send_event(AppEvent::SavePng {});
        }) as Box<dyn Fn()>);

        let save_png_button = document.unwrap().get_element_by_id("save_png_button").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        save_png_button.set_onclick(Some(save_png_function.as_ref().unchecked_ref()));

        Self
        {
            event_loop,
            event_loop_proxy,
            canvas_window,
            window_size,

            document,

            save_png_function
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub fn new_native() -> Self
    {
        let event_loop: EventLoop<AppEvent> = EventLoop::with_user_event();
        let event_loop_proxy                = Rc::new(event_loop.create_proxy());

        let canvas_window = WindowBuilder::new().build(&event_loop).unwrap();
        let window_size   = canvas_window.inner_size();

        Self
        {
            event_loop,
            event_loop_proxy,
            canvas_window,
            window_size
        }
    }

    pub async fn run(self)
    {
        let canvas_window = self.canvas_window;
        let event_loop    = self.event_loop;

        let mut window_size = self.window_size;

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
                state.update();
                match state.render()
                {
                    Ok(_) =>
                    {
                    }

                    Err(wgpu::SwapChainError::Lost) =>
                    {
                        state.resize(&window_size);
                    }

                    Err(wgpu::SwapChainError::OutOfMemory) =>
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
                            match state.get_png_data()
                            {
                                Ok(mut image_array) =>
                                {
                                    let document = self.document;

                                    let image_data = web_sys::ImageData::new_with_u8_clamped_array(Clamped(image_array.as_mut_slice()), state.board_size.width).unwrap();

                                    let canvas = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
                                    canvas.set_width(state.board_size.width);
                                    canvas.set_height(state.board_size.height);

                                    let canvas_context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
                                    canvas_context.put_image_data(&image_data, 0.0, 0.0);

                                    let link = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
                                    link.set_target(&canvas.to_data_url_with_type("image/png").unwrap());
                                    link.set_download(&"LightsOutMatrix.png");
                                    link.click();

                                    link.remove();
                                    canvas.remove();
                                },

                                Err(message) =>
                                {
                                    web_sys::console::log_1(&format!("Error saving PNG image: {}", message).into());
                                }
                            }
                        }
                    }
                }
            }

            _ => {}
        });
    }
}