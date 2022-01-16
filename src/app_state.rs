use winit::
{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop, EventLoopProxy},
    window::{WindowBuilder, Window},
};

#[cfg(target_arch = "wasm32")]
use
{
    winit::platform::web::WindowBuilderExtWebSys,
    wasm_bindgen::closure::Closure,
    wasm_bindgen::{JsCast, Clamped}
};

use std::rc::Rc;
use super::stafra_state;
use crate::stafra_state::{StafraState, StandardResetBoardType};

#[derive(Copy, Clone, PartialEq)]
enum RunState
{
    Stopped,
    Paused,
    Running
}

enum ResetOption
{
    Corners,
    Sides,
    Center,
    Custom
}

pub struct AppState
{
    event_loop:       EventLoop<AppEvent>,
    event_loop_proxy: Rc<EventLoopProxy<AppEvent>>,
    canvas_window:    Window,
    window_size:      winit::dpi::PhysicalSize<u32>,
    run_state:        RunState,

    #[cfg(target_arch = "wasm32")]
    document: web_sys::Document,

    #[cfg(target_arch = "wasm32")]
    board_upload_image_function: Closure<dyn Fn(web_sys::Event)>,

    #[cfg(target_arch = "wasm32")]
    board_upload_file_function: Closure<dyn Fn(web_sys::Event)>,

    #[cfg(target_arch = "wasm32")]
    save_png_function: Closure<dyn Fn()>,

    #[cfg(target_arch = "wasm32")]
    play_pause_function: Closure<dyn Fn()>,

    #[cfg(target_arch = "wasm32")]
    stop_function: Closure<dyn Fn()>,

    #[cfg(target_arch = "wasm32")]
    next_frame_function: Closure<dyn Fn()>,

    #[cfg(target_arch = "wasm32")]
    reset_board_function: Closure<dyn Fn(web_sys::Event)>,

    #[cfg(target_arch = "wasm32")]
    reset_board_custom_function: Closure<dyn Fn(web_sys::Event)>,
}

enum AppEvent
{
    SavePng {},
    SwitchPlayPause {},
    Stop {},
    NextFrame {},
    ResetBoard { reset_option: ResetOption },
    ResetBoardCustom { image_data: web_sys::ImageData },
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


        let event_loop_next_frame = event_loop_proxy.clone();
        let next_frame_function = Closure::wrap(Box::new(move ||
        {
            event_loop_next_frame.send_event(AppEvent::NextFrame {});
        }) as Box<dyn Fn()>);

        let next_frame_button = document.get_element_by_id("button_next_frame").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        next_frame_button.set_onclick(Some(next_frame_function.as_ref().unchecked_ref()));


        let document_board_upload = document.clone();
        let event_loop_proxy_upload_board = event_loop_proxy.clone();
        let board_upload_image_function = Closure::wrap(Box::new(move |event: web_sys::Event|
        {
            let board_image = event.target().unwrap().dyn_into::<web_sys::HtmlImageElement>().unwrap();

            let canvas_board   = document_board_upload.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
            let canvas_context = canvas_board.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

            canvas_board.set_width(board_image.width());
            canvas_board.set_height(board_image.height());

            canvas_context.draw_image_with_html_image_element(&board_image, 0.0, 0.0);
            let image_data = canvas_context.get_image_data(0.0, 0.0, board_image.width() as f64, board_image.height() as f64).unwrap();

            event_loop_proxy_upload_board.send_event(AppEvent::ResetBoardCustom {image_data});

            canvas_board.remove();
        }) as Box<dyn Fn(web_sys::Event)>);

        let board_upload_image = web_sys::HtmlImageElement::new().unwrap();
        board_upload_image.set_onload(Some(board_upload_image_function.as_ref().unchecked_ref()));

        let board_upload_file_function = Closure::wrap(Box::new(move |event: web_sys::Event|
        {
            board_upload_image.set_src(&event.target().unwrap().dyn_into::<web_sys::FileReader>().unwrap().result().unwrap().as_string().unwrap());
        }) as Box<dyn Fn(web_sys::Event)>);

        let board_upload_reader = web_sys::FileReader::new().unwrap();
        board_upload_reader.set_onload(Some(board_upload_file_function.as_ref().unchecked_ref()));

        let board_reset_select_with_custom = document.get_element_by_id("initial_states").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();

        let mut board_reset_custom_option_index   = 0;
        while let Some(board_reset_custom_option) = board_reset_select_with_custom.options().item(board_reset_custom_option_index)
        {
            let option_element = board_reset_custom_option.dyn_into::<web_sys::HtmlOptionElement>().unwrap();
            if option_element.value() == "initial_state_custom_value"
            {
                break;
            }

            board_reset_custom_option_index  += 1;
        }

        let reset_board_custom_function = Closure::wrap(Box::new(move |event: web_sys::Event|
        {
            let input_files = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().files().unwrap();
            if input_files.length() > 0
            {
                let image_file = input_files.item(0).unwrap();

                let filename = image_file.name();
                let custom_option_element = board_reset_select_with_custom.options().item(board_reset_custom_option_index).unwrap().dyn_into::<web_sys::HtmlOptionElement>().unwrap();
                custom_option_element.set_text(&filename);

                board_reset_select_with_custom.set_selected_index(board_reset_custom_option_index as i32);
                board_upload_reader.read_as_data_url(&image_file);
            }
        }) as Box<dyn Fn(web_sys::Event)>);

        let initial_state_upload_input = document.get_element_by_id("board_input").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        initial_state_upload_input.set_onchange(Some(reset_board_custom_function.as_ref().unchecked_ref()));


        let event_loop_reset_board = event_loop_proxy.clone();
        let reset_board_function = Closure::wrap(Box::new(move |event: web_sys::Event|
        {
            match event.target().unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap().value().as_str()
            {
                "initial_state_corners" => {event_loop_reset_board.send_event(AppEvent::ResetBoard {reset_option: ResetOption::Corners});}
                "initial_state_sides"   => {event_loop_reset_board.send_event(AppEvent::ResetBoard {reset_option: ResetOption::Sides});}
                "initial_state_center"  => {event_loop_reset_board.send_event(AppEvent::ResetBoard {reset_option: ResetOption::Center});}
                "initial_state_custom"  => {event_loop_reset_board.send_event(AppEvent::ResetBoard {reset_option: ResetOption::Custom});}
                _ => {}
            };
        }) as Box<dyn Fn(web_sys::Event)>);

        let initial_state_select = document.get_element_by_id("initial_states").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        initial_state_select.set_onchange(Some(reset_board_function.as_ref().unchecked_ref()));

        let window_size = canvas_window.inner_size();
        let run_state   = RunState::Running;

        Self
        {
            event_loop,
            event_loop_proxy,
            canvas_window,
            window_size,
            run_state,

            document,

            board_upload_image_function,
            board_upload_file_function,
            save_png_function,
            play_pause_function,
            stop_function,
            next_frame_function,
            reset_board_function,
            reset_board_custom_function
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

        let run_state = RunState::Running;
        Self
        {
            event_loop,
            event_loop_proxy,
            canvas_window,
            window_size,
            run_state
        }
    }

    pub async fn run(self)
    {
        #[cfg(target_arch = "wasm32")]
        let document = self.document;

        let canvas_window = self.canvas_window;
        let event_loop    = self.event_loop;

        let mut window_size = self.window_size;
        let mut run_state   = self.run_state;

        let mut main_state = stafra_state::StafraState::new(&canvas_window, 1023, 1023).await;
        AppState::reset_board_standard(&mut main_state, &canvas_window, StandardResetBoardType::Corners);

        #[cfg(target_arch = "wasm32")]
        AppState::update_ui(&document, run_state);

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
                    main_state.resize(&window_size);
                }

                WindowEvent::ScaleFactorChanged {new_inner_size, ..} =>
                {
                    window_size = **new_inner_size;
                    main_state.resize(&window_size);
                }

                _ => {}
            },

            Event::RedrawRequested(_) =>
            {
                if run_state == RunState::Running
                {
                    main_state.update();
                }

                #[cfg(target_arch = "wasm32")]
                match main_state.check_png_data_request()
                {
                    Ok((mut image_array, width, height, row_pitch)) =>
                    {
                        let image_data = web_sys::ImageData::new_with_u8_clamped_array(Clamped(image_array.as_mut_slice()), row_pitch).unwrap();

                        let canvas = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
                        canvas.set_width(width);
                        canvas.set_height(height);

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

                match main_state.render()
                {
                    Ok(_) =>
                    {
                    }

                    Err(wgpu::SurfaceError::Lost) =>
                    {
                        main_state.resize(&window_size);
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
                            main_state.post_png_data_request();
                        }
                    }

                    AppEvent::SwitchPlayPause {} =>
                    {
                        if run_state == RunState::Running
                        {
                            run_state = RunState::Paused;
                        }
                        else
                        {
                            run_state = RunState::Running;
                        }

                        #[cfg(target_arch = "wasm32")]
                        AppState::update_ui(&document, run_state);
                    }

                    AppEvent::Stop {} =>
                    {
                        run_state = RunState::Stopped;

                        #[cfg(target_arch = "wasm32")]
                        AppState::update_ui(&document, run_state);

                        AppState::reset_board_unchanged(&mut main_state, &canvas_window);
                    },

                    AppEvent::NextFrame {} =>
                    {
                        if run_state != RunState::Running
                        {
                            main_state.update();
                        }
                    }

                    AppEvent::ResetBoard {reset_option} =>
                    {
                        #[cfg(target_arch = "wasm32")]
                        {
                            match reset_option
                            {
                                ResetOption::Corners =>
                                {
                                    AppState::reset_board_standard(&mut main_state, &canvas_window, StandardResetBoardType::Corners);
                                    run_state = RunState::Running;
                                },

                                ResetOption::Sides =>
                                {
                                    AppState::reset_board_standard(&mut main_state, &canvas_window, StandardResetBoardType::Edges);
                                    run_state = RunState::Running;
                                },

                                ResetOption::Center =>
                                {
                                    AppState::reset_board_standard(&mut main_state, &canvas_window, StandardResetBoardType::Center);
                                    run_state = RunState::Running;
                                },

                                ResetOption::Custom =>
                                {
                                    let initial_state_upload_input = document.get_element_by_id("board_input").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
                                    initial_state_upload_input.click();
                                }
                            }

                            AppState::update_ui(&document, run_state);
                        }
                    }

                    AppEvent::ResetBoardCustom {image_data} =>
                    {
                        #[cfg(target_arch = "wasm32")]
                        {
                            AppState::reset_board_custom(&mut main_state, &canvas_window, image_data);

                            run_state = RunState::Running;

                            AppState::update_ui(&document, run_state);
                        }
                    }
                }
            }

            _ => {}
        });
    }

    #[cfg(target_arch = "wasm32")]
    fn update_ui(document: &web_sys::Document, run_state: RunState)
    {
        let play_pause_button = document.get_element_by_id("button_play_pause").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        if run_state == RunState::Running
        {
            play_pause_button.set_text_content(Some("⏸️"));
        }
        else
        {
            play_pause_button.set_text_content(Some("▶️"));
        }

        let next_frame_button = document.get_element_by_id("button_next_frame").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        next_frame_button.set_disabled(run_state == RunState::Running);

        let initial_board_select = document.get_element_by_id("initial_states").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        initial_board_select.set_disabled(run_state != RunState::Stopped);
    }

    fn reset_board_unchanged(state: &mut StafraState, window: &Window)
    {
        state.reset_board_unchanged();
        window.request_redraw();
    }

    fn reset_board_standard(state: &mut StafraState, window: &Window, reset_type: StandardResetBoardType)
    {
        state.reset_board_standard(reset_type);
        window.request_redraw();
    }

    #[cfg(target_arch = "wasm32")]
    fn reset_board_custom(state: &mut StafraState, window: &Window, image_data: web_sys::ImageData)
    {
        state.reset_board_custom(image_data.data().to_vec(), image_data.width(), image_data.height());
        window.request_redraw();
    }
}