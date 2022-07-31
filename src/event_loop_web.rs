#![cfg(target_arch = "wasm32")]

use
{
    wasm_bindgen::closure::Closure,
    wasm_bindgen::{JsCast, JsValue, Clamped},
    std::rc::Rc,
    std::cell::RefCell
};

use super::stafra_state;
use super::app_state;
use super::video_record_state;

use crate::app_state::RunState;

struct QueryStringParams
{
    initial_state: stafra_state::StandardResetBoardType,
    size_index:    u32,

    final_frame: u32,

    spawn:            u32,
    smooth_transform: bool,

    click_rule_data: app_state::ClickRuleInitData,
}

pub async fn run_event_loop()
{
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();

    let main_canvas       = document.get_element_by_id("stafra_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let click_rule_canvas = document.get_element_by_id("click_rule_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    //Apparently it's necessary to do it, width and height are not set up automatically
    main_canvas.set_width((main_canvas.client_width()               as f64 * window.device_pixel_ratio()) as u32);
    main_canvas.set_height((main_canvas.client_height()             as f64 * window.device_pixel_ratio()) as u32);
    click_rule_canvas.set_width((click_rule_canvas.client_width()   as f64 * window.device_pixel_ratio()) as u32);
    click_rule_canvas.set_height((click_rule_canvas.client_height() as f64 * window.device_pixel_ratio()) as u32);

    //Reading query string data
    let state_params = parse_query_string(window.location().search().unwrap().as_str());
    setup_initial_ui(&state_params);

    //Initializing the state
    let board_size = app_state::AppState::board_size_from_index(state_params.size_index);

    let app_state_rc          = Rc::new(RefCell::new(app_state::AppState::new(state_params.click_rule_data, state_params.final_frame)));
    let stafra_state_rc       = Rc::new(RefCell::new(stafra_state::StafraState::new_web(&main_canvas, &click_rule_canvas, board_size, board_size).await));
    let video_record_state_rc = Rc::new(RefCell::new(video_record_state::VideoRecordState::new()));

    let mut app_state      = app_state_rc.borrow_mut();
    let mut stafra_state   = stafra_state_rc.borrow_mut();
    let video_record_state = video_record_state_rc.borrow();

    if !video_record_state.is_recording_supported()
    {
        web_sys::console::warn_1(&"Warning: this browser does not support video recording with WebCodecs".into());
    }

    stafra_state.reset_board_standard(state_params.initial_state);
    stafra_state.reset_click_rule(&app_state.click_rule_data);
    stafra_state.set_spawn_period(state_params.spawn);
    stafra_state.set_smooth_transform_enabled(state_params.smooth_transform);
    stafra_state.clear_restriction();

    //Setting closures
    create_closures(app_state_rc.clone(), stafra_state_rc.clone(), video_record_state_rc.clone());

    //Refresh handler
    let app_state_clone_for_refresh          = app_state_rc.clone();
    let stafra_state_clone_for_refresh       = stafra_state_rc.clone();
    let video_record_state_clone_for_refresh = video_record_state_rc.clone();

    //Apparently the only way to detect canvas resize is to keep track of its size and compare it to the actual one each frame
    let mut current_main_canvas_width        = main_canvas.width();
    let mut current_main_canvas_height       = main_canvas.height();
    let mut current_click_rule_canvas_width  = click_rule_canvas.width();
    let mut current_click_rule_canvas_height = click_rule_canvas.height();

    let refresh_function: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let refresh_function_copy = refresh_function.clone();
    *refresh_function_copy.borrow_mut() = Some(Closure::wrap(Box::new(move ||
    {
        let mut app_state      = app_state_clone_for_refresh.borrow_mut();
        let mut stafra_state   = stafra_state_clone_for_refresh.borrow_mut();
        let video_record_state = video_record_state_clone_for_refresh.borrow_mut();

        let window = web_sys::window().unwrap();

        if app_state.run_state == RunState::Running
        {
            stafra_state.update();
        }
        else if app_state.run_state == RunState::Recording && !stafra_state.video_frame_queue_full() && video_record_state.is_recording_supported()
        {
            let video_frame_channel = video_record_state.get_video_frame_channel();

            stafra_state.update();
            stafra_state.post_video_frame_request(move |pixel_data, width, height|
            {
                video_frame_channel.send(video_record_state::VideoFrameData{pixel_data, width, height}).unwrap();
            });
        }
        else if app_state.run_state == RunState::PausedRecording && video_record_state.is_recording_supported()
        {
            update_next_frame_button_paused_recording(!stafra_state.video_frame_queue_full());
        }

        if app_state.last_frame == stafra_state.frame_number() && app_state.run_state != RunState::Paused
        {
            app_state.run_state = RunState::Paused;
            update_ui(app_state.run_state);
        }

        //Update and resize surfaces
        let main_canvas_width  = main_canvas.width();
        let main_canvas_height = main_canvas.height();
        if main_canvas_width != current_main_canvas_width || main_canvas_height != current_main_canvas_height
        {
            current_main_canvas_width  = main_canvas_width;
            current_main_canvas_height = main_canvas_height;

            stafra_state.resize(current_main_canvas_width as u32, current_main_canvas_height as u32);
        }

        let click_rule_canvas_width  = click_rule_canvas.width();
        let click_rule_canvas_height = click_rule_canvas.height();
        if click_rule_canvas_width != current_click_rule_canvas_width || click_rule_canvas_height != current_click_rule_canvas_height
        {
            current_click_rule_canvas_width  = click_rule_canvas_width;
            current_click_rule_canvas_height = click_rule_canvas_height;

            stafra_state.resize_click_rule(current_click_rule_canvas_width as u32, current_click_rule_canvas_height as u32);
        }

        stafra_state.update_visual_info();

        //Display state
        match stafra_state.render()
        {
            Ok(_) =>
            {
            }

            Err(wgpu::SurfaceError::Lost) =>
            {
                let document = window.document().unwrap();
                let main_canvas = document.get_element_by_id("stafra_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
                stafra_state.resize(main_canvas.client_width() as u32, main_canvas.client_height() as u32);
            }

            Err(wgpu::SurfaceError::OutOfMemory) =>
            {
                //Break the control flow
                refresh_function.take();
                return;
            }

            Err(error) =>
            {
                web_sys::console::log_1(&format!("{:?}", error).into());
            }
        }

        window.request_animation_frame(refresh_function.borrow().as_ref().unwrap().as_ref().unchecked_ref()).expect("Request animation frame error");
    }) as Box<dyn FnMut()>));


    //Start
    app_state.run_state = RunState::Running;

    stafra_state.set_click_rule_read_only(true);
    stafra_state.set_click_rule_grid_enabled(false);

    update_ui(app_state.run_state);

    window.request_animation_frame(refresh_function_copy.borrow().as_ref().unwrap().as_ref().unchecked_ref()).expect("Request animation frame error!");
}

fn create_closures(app_state_rc: Rc<RefCell<app_state::AppState>>, stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>, video_record_state_rc: Rc<RefCell<video_record_state::VideoRecordState>>)
{
    create_click_rule_change_closure(app_state_rc.clone(), stafra_state_rc.clone());

    create_save_png_closure(stafra_state_rc.clone());

    create_play_pause_closure(app_state_rc.clone(), stafra_state_rc.clone(), video_record_state_rc.clone());
    create_stop_closure(app_state_rc.clone(), stafra_state_rc.clone(), video_record_state_rc.clone());
    create_next_frame_closure(app_state_rc.clone(), stafra_state_rc.clone(), video_record_state_rc.clone());

    create_enable_last_frame_closure(app_state_rc.clone(), video_record_state_rc.clone());
    create_change_last_frame_closure(app_state_rc.clone(), video_record_state_rc.clone());

    create_enable_spawn_closure(stafra_state_rc.clone());
    create_decrement_spawn_closure(stafra_state_rc.clone());
    create_increment_spawn_closure(stafra_state_rc.clone());
    create_change_spawn_closure(stafra_state_rc.clone());

    create_change_smooth_transform_closure(stafra_state_rc.clone());

    create_show_grid_closure(stafra_state_rc.clone());

    create_upload_restriction_closure();
    create_clear_restriction_closure(stafra_state_rc.clone());

    create_board_upload_input_closure(app_state_rc.clone(), stafra_state_rc.clone());
    create_upload_restriction_input_closure(stafra_state_rc.clone());

    create_select_initial_state_closure(stafra_state_rc.clone());
    create_select_size_closure(app_state_rc.clone(), stafra_state_rc.clone());
}

fn create_click_rule_change_closure(app_state_rc: Rc<RefCell<app_state::AppState>>, stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let click_rule_canvas = document.get_element_by_id("click_rule_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let click_rule_change_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut app_state    = app_state_rc.borrow_mut();
        let mut stafra_state = stafra_state_rc.borrow_mut();

        if app_state.run_state == RunState::Stopped
        {
            let window       = web_sys::window().unwrap();
            let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

            let mouse_event       = event.dyn_into::<web_sys::MouseEvent>().unwrap();
            let click_rule_canvas = mouse_event.target().unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

            let click_x = mouse_event.page_x() - click_rule_canvas.offset_left();
            let click_y = mouse_event.page_y() - click_rule_canvas.offset_top();

            let canvas_width  = click_rule_canvas.client_width();
            let canvas_height = click_rule_canvas.client_height();

            let x_normalized = (click_x as f32) / (canvas_width  as f32);
            let y_normalized = (click_y as f32) / (canvas_height as f32);

            let click_rule_size = 32;
            let edit_index_x_unrestricted = (x_normalized * (click_rule_size as f32)) as i32;
            let edit_index_y_unrestricted = (y_normalized * (click_rule_size as f32)) as i32;

            let edit_index_x = edit_index_x_unrestricted.clamp(0, click_rule_size - 1);
            let edit_index_y = edit_index_y_unrestricted.clamp(0, click_rule_size - 1);

            let click_rule_index = (edit_index_y * click_rule_size + edit_index_x) as usize;

            let current_cell_state = app_state.click_rule_data[click_rule_index] != 0;
            app_state.click_rule_data[click_rule_index] = (!current_cell_state) as u8;

            stafra_state.reset_click_rule(&app_state.click_rule_data);
            query_string.set("click_rule", &app_state.encode_click_rule_base64());

            let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
            window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();
        }
    })
    as Box<dyn Fn(web_sys::Event)>);

    click_rule_canvas.set_onmousedown(Some(click_rule_change_closure.as_ref().unchecked_ref()));
    click_rule_change_closure.forget();
}

fn create_save_png_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let save_png_button = document.get_element_by_id("button_save_png").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let save_png_closure = Closure::wrap(Box::new(move ||
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();
        stafra_state.post_save_png_request(move |pixel_data, width, height|
        {
            let image_data = web_sys::ImageData::new_with_u8_clamped_array_and_sh(Clamped(pixel_data.as_slice()), width, height).unwrap();
            save_image_data(image_data);
        });
    })
    as Box<dyn Fn()>);

    save_png_button.set_onclick(Some(save_png_closure.as_ref().unchecked_ref()));
    save_png_closure.forget();
}

fn create_play_pause_closure(app_state_rc: Rc<RefCell<app_state::AppState>>, stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>, video_record_state_rc: Rc<RefCell<video_record_state::VideoRecordState>>)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let play_pause_button = document.get_element_by_id("button_play_pause").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let play_pause_closure = Closure::wrap(Box::new(move ||
    {
        let mut app_state      = app_state_rc.borrow_mut();
        let mut stafra_state   = stafra_state_rc.borrow_mut();
        let video_record_state = video_record_state_rc.borrow();

        stafra_state.set_click_rule_read_only(true);

        if video_record_state.is_recording_supported()
        {
            app_state.run_state = match app_state.run_state
            {
                RunState::Stopped | RunState::Paused => RunState::Running,
                RunState::Running                    => RunState::Paused,
                RunState::Recording                  => RunState::PausedRecording,
                RunState::PausedRecording            => RunState::Recording
            };
        }
        else
        {
            app_state.run_state = match app_state.run_state
            {
                RunState::Stopped | RunState::Paused => RunState::Running,
                RunState::Running                    => RunState::Paused,
                RunState::Recording                  => RunState::Paused,
                RunState::PausedRecording            => RunState::Running
            };
        }

        update_ui(app_state.run_state);

        if app_state.run_state == RunState::PausedRecording
        {
            update_next_frame_button_paused_recording(!stafra_state.video_frame_queue_full());
        }

    }) as Box<dyn Fn()>);

    play_pause_button.set_onclick(Some(play_pause_closure.as_ref().unchecked_ref()));
    play_pause_closure.forget();
}

fn create_stop_closure(app_state_rc: Rc<RefCell<app_state::AppState>>, stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>, video_record_state_rc: Rc<RefCell<video_record_state::VideoRecordState>>)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let stop_button = document.get_element_by_id("button_stop_record").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let stop_closure = Closure::wrap(Box::new(move ||
    {
        let mut app_state          = app_state_rc.borrow_mut();
        let mut stafra_state       = stafra_state_rc.borrow_mut();
        let mut video_record_state = video_record_state_rc.borrow_mut();

        match app_state.run_state
        {
            RunState::Stopped =>
            {
                if !video_record_state.pending() && video_record_state.is_recording_supported()
                {
                    video_record_state.restart().unwrap();
                    video_record_state.set_frame_limit(app_state.last_frame);
                    app_state.run_state = RunState::Recording;
                }
            }

            RunState::Recording | RunState::PausedRecording =>
            {
                video_record_state.set_frame_limit(stafra_state.frame_number());
                app_state.run_state = RunState::Stopped;

                stafra_state.reset_board_unchanged();
                stafra_state.set_click_rule_read_only(false);
            }

            RunState::Paused | RunState::Running =>
            {
                app_state.run_state = RunState::Stopped;

                stafra_state.reset_board_unchanged();
                stafra_state.set_click_rule_read_only(false);
            }
        };

        update_ui(app_state.run_state);
    }) as Box<dyn Fn()>);

    stop_button.set_onclick(Some(stop_closure.as_ref().unchecked_ref()));
    stop_closure.forget();
}

fn create_next_frame_closure(app_state_rc: Rc<RefCell<app_state::AppState>>, stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>, video_record_state_rc: Rc<RefCell<video_record_state::VideoRecordState>>)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let next_frame_button = document.get_element_by_id("button_next_frame").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let next_frame_closure = Closure::wrap(Box::new(move ||
    {
        let mut app_state      = app_state_rc.borrow_mut();
        let mut stafra_state   = stafra_state_rc.borrow_mut();
        let video_record_state = video_record_state_rc.borrow_mut();

        match app_state.run_state
        {
            RunState::PausedRecording => if !stafra_state.video_frame_queue_full() && video_record_state.is_recording_supported()
            {
                let video_frame_channel = video_record_state.get_video_frame_channel();

                stafra_state.update();
                stafra_state.post_video_frame_request(move |pixel_data: Vec<u8>, width: u32, height: u32|
                {
                    video_frame_channel.send(video_record_state::VideoFrameData{pixel_data, width, height}).unwrap();
                });

                update_next_frame_button_paused_recording(!stafra_state.video_frame_queue_full());
                if app_state.last_frame == stafra_state.frame_number()
                {
                    app_state.run_state = RunState::Paused;
                }
            }

            RunState::Paused =>
            {
                stafra_state.update();
            }

            RunState::Stopped =>
            {
                app_state.run_state = RunState::Paused;
                update_ui(app_state.run_state);

                //Update twice to properly initialize the first frame
                stafra_state.update();
                stafra_state.update();
            }

            _ => {}
        }
    }) as Box<dyn Fn()>);

    next_frame_button.set_onclick(Some(next_frame_closure.as_ref().unchecked_ref()));
    next_frame_closure.forget();
}

fn create_enable_last_frame_closure(app_state_rc: Rc<RefCell<app_state::AppState>>, video_record_state_rc: Rc<RefCell<video_record_state::VideoRecordState>>)
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let last_frame_checkbox = document.get_element_by_id("last_frame_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let enable_last_frame_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut app_state          = app_state_rc.borrow_mut();
        let mut video_record_state = video_record_state_rc.borrow_mut();

        let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

        let last_frame_input = document.get_element_by_id("last_frame_number").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

        let last_frame_checkbox = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        if last_frame_checkbox.checked()
        {
            let last_frame = last_frame_input.value_as_number();
            if !last_frame.is_nan()
            {
                app_state.last_frame = last_frame as u32;
                query_string.set("final_frame", &last_frame.to_string());
            }
            else
            {
                app_state.last_frame = u32::MAX;
                query_string.delete("final_frame");
            }

            last_frame_input.set_disabled(false);
        }
        else
        {
            app_state.last_frame = u32::MAX;
            last_frame_input.set_disabled(true);

            query_string.delete("final_frame");
        }

        let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
        window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();

        video_record_state.set_frame_limit(app_state.last_frame);

    }) as Box<dyn Fn(web_sys::Event)>);

    last_frame_checkbox.set_onclick(Some(enable_last_frame_closure.as_ref().unchecked_ref()));
    enable_last_frame_closure.forget();
}

fn create_enable_spawn_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let spawn_checkbox = document.get_element_by_id("spawn_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let enable_spawn_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

        let spawn_decrement_button    = document.get_element_by_id("decrement_spawn").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        let spawn_increment_button    = document.get_element_by_id("increment_spawn").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        let spawn_slider              = document.get_element_by_id("spawn_range").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let smooth_transform_checkbox = document.get_element_by_id("smooth_transform_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

        let spawn_checkbox = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

        spawn_decrement_button.set_disabled(!spawn_checkbox.checked());
        spawn_increment_button.set_disabled(!spawn_checkbox.checked());
        spawn_slider.set_disabled(!spawn_checkbox.checked());
        smooth_transform_checkbox.set_disabled(!spawn_checkbox.checked());

        if spawn_checkbox.checked()
        {
            let spawn_period_input = document.get_element_by_id("spawn_number").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
            let spawn_period = spawn_period_input.value().parse::<u32>().expect("Not a number");

            stafra_state.set_spawn_period(spawn_period);
            query_string.set("spawn_period", &spawn_period.to_string());
        }
        else
        {
            stafra_state.set_spawn_period(u32::MAX);
            query_string.delete("spawn_period");
        }

        let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
        window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();

    }) as Box<dyn Fn(web_sys::Event)>);

    spawn_checkbox.set_onclick(Some(enable_spawn_closure.as_ref().unchecked_ref()));
    enable_spawn_closure.forget();
}

fn create_decrement_spawn_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let decrement_spawn_button = document.get_element_by_id("decrement_spawn").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let decrement_spawn_closure = Closure::wrap(Box::new(move ||
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

        let spawn_period_input = document.get_element_by_id("spawn_number").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

        let spawn_period_slider = document.get_element_by_id("spawn_range").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let new_spawn_period = ((spawn_period_input.value().parse::<u32>().expect("Not a number")) - 1).clamp(1, 255);

        let spawn_period_string = new_spawn_period.to_string();

        spawn_period_input.set_value(&spawn_period_string);
        spawn_period_slider.set_value(&spawn_period_string);

        query_string.set("spawn_period", &spawn_period_string);
        stafra_state.set_spawn_period(new_spawn_period);

        let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
        window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();

    }) as Box<dyn Fn()>);

    decrement_spawn_button.set_onclick(Some(decrement_spawn_closure.as_ref().unchecked_ref()));
    decrement_spawn_closure.forget();
}

fn create_increment_spawn_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let increment_spawn_button = document.get_element_by_id("increment_spawn").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let increment_spawn_closure = Closure::wrap(Box::new(move ||
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

        let spawn_period_input = document.get_element_by_id("spawn_number").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

        let spawn_period_slider = document.get_element_by_id("spawn_range").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let new_spawn_period = ((spawn_period_input.value().parse::<u32>().expect("Not a number")) + 1).clamp(1, 255);

        let spawn_period_string = new_spawn_period.to_string();

        spawn_period_input.set_value(&spawn_period_string);
        spawn_period_slider.set_value(&spawn_period_string);

        query_string.set("spawn_period", &spawn_period_string);
        stafra_state.set_spawn_period(new_spawn_period);

        let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
        window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();

    }) as Box<dyn Fn()>);

    increment_spawn_button.set_onclick(Some(increment_spawn_closure.as_ref().unchecked_ref()));
    increment_spawn_closure.forget();
}

fn create_change_spawn_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let spawn_range = document.get_element_by_id("spawn_range").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let spawn_query_string_timeout_closure = Closure::wrap(Box::new(move |spawn_value: &JsValue|
    {
        let window = web_sys::window().unwrap();
        let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

        query_string.set("spawn_period", &spawn_value.as_string().unwrap());

        let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
        window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();

    }) as Box<dyn FnMut(&JsValue)>);

    let timeout_update_arguments = js_sys::Array::new_with_length(1);
    let change_spawn_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let spawn_period_slider = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

        let spawn_period_input = document.get_element_by_id("spawn_number").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let new_spawn_period = spawn_period_slider.value_as_number() as u32;

        spawn_period_input.set_value(&new_spawn_period.to_string());
        stafra_state.set_spawn_period(new_spawn_period);

        timeout_update_arguments.set(0, JsValue::from_str(&new_spawn_period.to_string()));
        window.set_timeout_with_callback_and_timeout_and_arguments(&spawn_query_string_timeout_closure.as_ref().unchecked_ref(), 50, &timeout_update_arguments).unwrap();

    }) as Box<dyn Fn(web_sys::Event)>);

    spawn_range.set_oninput(Some(change_spawn_closure.as_ref().unchecked_ref()));
    change_spawn_closure.forget();
}

fn create_change_smooth_transform_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let window   = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let smooth_transform_checkbox = document.get_element_by_id("smooth_transform_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let smooth_transform_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

        let smooth_transform_checkbox = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        stafra_state.set_smooth_transform_enabled(smooth_transform_checkbox.checked());

        if smooth_transform_checkbox.checked()
        {
            query_string.set("smooth_transform", "y");
        }
        else
        {
            query_string.delete("smooth_transform");
        }

        let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
        window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();

    }) as Box<dyn Fn(web_sys::Event)>);

    smooth_transform_checkbox.set_onclick(Some(smooth_transform_closure.as_ref().unchecked_ref()));
    smooth_transform_closure.forget();
}

fn create_change_last_frame_closure(app_state_rc: Rc<RefCell<app_state::AppState>>, video_record_state_rc: Rc<RefCell<video_record_state::VideoRecordState>>)
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let last_frame_input = document.get_element_by_id("last_frame_number").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let change_last_frame_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut app_state          = app_state_rc.borrow_mut();
        let mut video_record_state = video_record_state_rc.borrow_mut();

        let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

        let last_frame_input = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        let new_value = last_frame_input.value_as_number();
        if !new_value.is_nan()
        {
            app_state.last_frame = new_value as u32;
            query_string.set("final_frame", &new_value.to_string());
        }
        else
        {
            app_state.last_frame = u32::MAX;
            query_string.delete("final_frame");
        }

        let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
        window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();

        video_record_state.set_frame_limit(app_state.last_frame);

    }) as Box<dyn Fn(web_sys::Event)>);

    last_frame_input.set_oninput(Some(change_last_frame_closure.as_ref().unchecked_ref()));
    change_last_frame_closure.forget();
}

fn create_show_grid_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let show_grid_checkbox = document.get_element_by_id("grid_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let show_grid_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let show_grid_checkbox = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        stafra_state.set_click_rule_grid_enabled(show_grid_checkbox.checked());
    }) as Box<dyn Fn(web_sys::Event)>);

    show_grid_checkbox.set_onclick(Some(show_grid_closure.as_ref().unchecked_ref()));
    show_grid_closure.forget();
}

fn create_board_upload_input_closure(app_state_rc: Rc<RefCell<app_state::AppState>>, stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let initial_state_upload_input = document.get_element_by_id("board_input").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let board_upload_image_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut app_state    = app_state_rc.borrow_mut();
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let board_image = event.target().unwrap().dyn_into::<web_sys::HtmlImageElement>().unwrap();

        let document       = web_sys::window().unwrap().document().unwrap();
        let canvas_board   = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let canvas_context = canvas_board.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

        canvas_board.set_width(board_image.width());
        canvas_board.set_height(board_image.height());

        canvas_context.draw_image_with_html_image_element(&board_image, 0.0, 0.0).expect("Draw image error!");
        let image_data = canvas_context.get_image_data(0.0, 0.0, board_image.width() as f64, board_image.height() as f64).unwrap();

        let new_size = stafra_state.reset_board_custom(image_data.data().to_vec(), image_data.width(), image_data.height());

        canvas_board.remove();


        let size_select = document.get_element_by_id("sizes").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        let size_index = (std::mem::size_of::<u32>() * 8) as u32 - new_size.leading_zeros() - 1;
        size_select.set_selected_index(size_index as i32);

        update_last_frame_with_size(new_size, &mut app_state);
    }) as Box<dyn Fn(web_sys::Event)>);

    let board_upload_image_element = web_sys::HtmlImageElement::new().unwrap();
    board_upload_image_element.set_onload(Some(board_upload_image_closure.as_ref().unchecked_ref()));

    let board_file_read_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let file_reader      = event.target().unwrap().dyn_into::<web_sys::FileReader>().unwrap();
        let file_read_result = file_reader.result().unwrap();

        let file_data = file_read_result.as_string().unwrap();
        board_upload_image_element.set_src(&file_data);
    }) as Box<dyn Fn(web_sys::Event)>);

    let board_file_reader = web_sys::FileReader::new().unwrap();
    board_file_reader.set_onload(Some(board_file_read_closure.as_ref().unchecked_ref()));

    board_file_read_closure.forget();
    board_upload_image_closure.forget();

    let initial_state_upload_input_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let input_files = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().files().unwrap();
        if input_files.length() > 0
        {
            let image_file = input_files.item(0).unwrap();
            let filename   = image_file.name();

            //Change the select option text
            let initial_state_select = document.get_element_by_id("initial_states").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
            let custom_option_index  = find_select_option_index(&initial_state_select, "initial_state_custom_value");

            let initial_state_options = initial_state_select.options();
            let custom_option_element = initial_state_options.item(custom_option_index.try_into().unwrap()).unwrap().dyn_into::<web_sys::HtmlOptionElement>().unwrap();

            custom_option_element.set_text(&filename);
            initial_state_select.set_selected_index(custom_option_index);

            //Read the data
            board_file_reader.read_as_data_url(&image_file).expect("Read data url error!");
        }
    }) as Box<dyn Fn(web_sys::Event)>);

    initial_state_upload_input.set_onchange(Some(initial_state_upload_input_closure.as_ref().unchecked_ref()));
    initial_state_upload_input_closure.forget();
}

fn create_upload_restriction_closure()
{
    let document = web_sys::window().unwrap().document().unwrap();
    let upload_restriction_button = document.get_element_by_id("button_upload_restriction").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let upload_restriction_closure = Closure::wrap(Box::new(move |_event: web_sys::Event|
    {
        let restriction_upload_input = document.get_element_by_id("restriction_input").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
        restriction_upload_input.click();

    }) as Box<dyn Fn(web_sys::Event)>);

    upload_restriction_button.set_onclick(Some(upload_restriction_closure.as_ref().unchecked_ref()));
    upload_restriction_closure.forget();
}

fn create_upload_restriction_input_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let restriction_upload_input = document.get_element_by_id("restriction_input").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let restriction_upload_image_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let restriction_image = event.target().unwrap().dyn_into::<web_sys::HtmlImageElement>().unwrap();

        let canvas_restriction = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let canvas_context     = canvas_restriction.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

        canvas_restriction.set_width(restriction_image.width());
        canvas_restriction.set_height(restriction_image.height());

        canvas_context.draw_image_with_html_image_element(&restriction_image, 0.0, 0.0).expect("Draw image error!");
        let image_data = canvas_context.get_image_data(0.0, 0.0, restriction_image.width() as f64, restriction_image.height() as f64).unwrap();

        stafra_state.upload_restriction(image_data.data().to_vec(), image_data.width(), image_data.height());

        canvas_restriction.remove();

        let clear_restriction_button = document.get_element_by_id("button_clear_restriction").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        clear_restriction_button.set_hidden(false);

    }) as Box<dyn Fn(web_sys::Event)>);

    let restriction_upload_image_element = web_sys::HtmlImageElement::new().unwrap();
    restriction_upload_image_element.set_onload(Some(restriction_upload_image_closure.as_ref().unchecked_ref()));

    let restriction_file_read_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let file_reader      = event.target().unwrap().dyn_into::<web_sys::FileReader>().unwrap();
        let file_read_result = file_reader.result().unwrap();

        let file_data = file_read_result.as_string().unwrap();
        restriction_upload_image_element.set_src(&file_data);
    }) as Box<dyn Fn(web_sys::Event)>);

    let restriction_file_reader = web_sys::FileReader::new().unwrap();
    restriction_file_reader.set_onload(Some(restriction_file_read_closure.as_ref().unchecked_ref()));

    restriction_file_read_closure.forget();
    restriction_upload_image_closure.forget();

    let restriction_upload_input_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let input_files = event.target().unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap().files().unwrap();
        if input_files.length() > 0
        {
            let image_file = input_files.item(0).unwrap();
            restriction_file_reader.read_as_data_url(&image_file).expect("Read data url error!");
        }
    }) as Box<dyn Fn(web_sys::Event)>);

    restriction_upload_input.set_onchange(Some(restriction_upload_input_closure.as_ref().unchecked_ref()));
    restriction_upload_input_closure.forget();
}

fn create_clear_restriction_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let document = web_sys::window().unwrap().document().unwrap();
    let clear_restriction_button  = document.get_element_by_id("button_clear_restriction").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let clear_restriction_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();
        stafra_state.clear_restriction();

        let clear_restriction_button = event.target().unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
        clear_restriction_button.set_hidden(true);
    }) as Box<dyn Fn(web_sys::Event)>);

    clear_restriction_button.set_onclick(Some(clear_restriction_closure.as_ref().unchecked_ref()));
    clear_restriction_closure.forget();
}

fn create_select_initial_state_closure(stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let initial_state_select = document.get_element_by_id("initial_states").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();

    let select_initial_state_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

        let board_reset_select = event.target().unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        match board_reset_select.value().as_str()
        {
            "initial_state_corners" =>
            {
                stafra_state.reset_board_standard(stafra_state::StandardResetBoardType::Corners);
                query_string.set("initial_state", "corners");
            },

            "initial_state_sides" =>
            {
                stafra_state.reset_board_standard(stafra_state::StandardResetBoardType::Edges);
                query_string.set("initial_state", "edges");
            },

            "initial_state_center" =>
            {
                stafra_state.reset_board_standard(stafra_state::StandardResetBoardType::Center);
                query_string.set("initial_state", "center");
            },

            "initial_state_custom" =>
            {
                let initial_state_upload_input = document.get_element_by_id("board_input").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
                initial_state_upload_input.click();

                query_string.delete("initial_state");
            },

            _ => {}
        }

        let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
        window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();

    }) as Box<dyn Fn(web_sys::Event)>);

    initial_state_select.set_onchange(Some(select_initial_state_closure.as_ref().unchecked_ref()));
    select_initial_state_closure.forget();
}

fn create_select_size_closure(app_state_rc: Rc<RefCell<app_state::AppState>>, stafra_state_rc: Rc<RefCell<stafra_state::StafraState>>)
{
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let size_select = document.get_element_by_id("sizes").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();

    let select_size_closure = Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut app_state    = app_state_rc.borrow_mut();
        let mut stafra_state = stafra_state_rc.borrow_mut();

        let query_string = web_sys::UrlSearchParams::new_with_str(window.location().search().unwrap().as_str()).unwrap();

        let size_select = event.target().unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        let board_size_selected_index = size_select.selected_index() as u32;

        let new_width  = app_state::AppState::board_size_from_index(board_size_selected_index);
        let new_height = app_state::AppState::board_size_from_index(board_size_selected_index);

        update_last_frame_with_size(std::cmp::min(new_width, new_height), &mut app_state);
        stafra_state.resize_board(new_width, new_height);

        query_string.set("size_index", &board_size_selected_index.to_string());

        let new_search_state = window.location().pathname().unwrap() + "?" + &query_string.to_string().as_string().unwrap();
        window.history().unwrap().replace_state_with_url(&JsValue::NULL, "", Some(&new_search_state)).unwrap();

    }) as Box<dyn Fn(web_sys::Event)>);

    size_select.set_onchange(Some(select_size_closure.as_ref().unchecked_ref()));
    select_size_closure.forget();
}

fn parse_query_string(query_string: &str) -> QueryStringParams
{
    let search_params = web_sys::UrlSearchParams::new_with_str(query_string).unwrap();

    let initial_state = match search_params.get("initial_state")
    {
        Some(value) => match value.to_lowercase().as_str()
        {
            "corners"         => stafra_state::StandardResetBoardType::Corners,
            "sides" | "edges" => stafra_state::StandardResetBoardType::Edges,
            "center"          => stafra_state::StandardResetBoardType::Center,
            _                 => stafra_state::StandardResetBoardType::Corners
        }

        None => stafra_state::StandardResetBoardType::Corners
    };

    let default_size_index = 9;  //Corresponds to 1023x1023
    let minimum_size_index = 0;  //Corresponds to 1x1
    let maximum_size_index = 13; //Corresponds to 16383x16383
    let size_index = match search_params.get("size_index")
    {
        Some(value) => value.parse::<u32>().unwrap_or(default_size_index).clamp(minimum_size_index, maximum_size_index),
        None        => default_size_index
    };

    let final_frame = match search_params.get("final_frame")
    {
        Some(value) => value.parse::<u32>().unwrap_or(u32::MAX).clamp(1, u32::MAX),
        None        => u32::MAX
    };

    let spawn = match search_params.get("spawn_period")
    {
        Some(value) => value.parse::<u32>().unwrap_or(u32::MAX).clamp(1, 255),
        None        => u32::MAX
    };

    let smooth_transform = match search_params.get("smooth_transform")
    {
        Some(value) => match value.to_lowercase().as_str()
        {
            "y" | "yes" | "1" | "true" => true,
            _                          => false
        }

        None => false
    };

    let click_rule_data = match search_params.get("click_rule")
    {
        Some(value) => app_state::ClickRuleInitData::Custom(app_state::parse_click_rule_base64(value.as_str())),
        None        => app_state::ClickRuleInitData::Default
    };

    QueryStringParams
    {
        initial_state,
        size_index,

        final_frame,

        spawn,
        smooth_transform,

        click_rule_data
    }
}

fn save_image_data(image_data: web_sys::ImageData)
{
    let document = web_sys::window().unwrap().document().unwrap();

    let canvas = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    canvas.set_width(image_data.width());
    canvas.set_height(image_data.height());

    let canvas_context = canvas.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();
    canvas_context.put_image_data(&image_data, 0.0, 0.0).expect("Image data put error!");

    let link = document.create_element("a").unwrap().dyn_into::<web_sys::HtmlAnchorElement>().unwrap();
    link.set_href(&canvas.to_data_url_with_type("image/png").unwrap());
    link.set_download(&"StabilityFractal.png");
    link.click();

    link.remove();
    canvas.remove();
}

fn find_select_option_index(select_element: &web_sys::HtmlSelectElement, value: &str) -> i32
{
    let options = select_element.options();

    let mut option_index: i32 = 0;
    while let Some(board_reset_custom_option) = options.item(option_index as u32)
    {
        let option_element = board_reset_custom_option.dyn_into::<web_sys::HtmlOptionElement>().unwrap();
        if option_element.value() == value
        {
            return option_index;
        }

        option_index += 1;
    }

    return -1;
}

fn setup_initial_ui(state_params: &QueryStringParams)
{
    let document = web_sys::window().unwrap().document().unwrap();

    let initial_state_select      = document.get_element_by_id("initial_states").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
    let size_select               = document.get_element_by_id("sizes").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
    let last_frame_checkbox       = document.get_element_by_id("last_frame_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let last_frame_input          = document.get_element_by_id("last_frame_number").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let spawn_checkbox            = document.get_element_by_id("spawn_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let smooth_transform_checkbox = document.get_element_by_id("smooth_transform_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let spawn_range               = document.get_element_by_id("spawn_range").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    let spawn_input               = document.get_element_by_id("spawn_number").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let board_size = app_state::AppState::board_size_from_index(state_params.size_index);

    initial_state_select.set_value(match state_params.initial_state
    {
        stafra_state::StandardResetBoardType::Corners => "initial_state_corners",
        stafra_state::StandardResetBoardType::Edges   => "initial_state_sides",
        stafra_state::StandardResetBoardType::Center  => "initial_state_center"
    });

    size_select.set_selected_index(state_params.size_index as i32);

    if state_params.final_frame != u32::MAX
    {
        last_frame_checkbox.set_checked(true);
        last_frame_input.set_value_as_number(state_params.final_frame as f64);
    }
    else
    {
        last_frame_checkbox.set_checked(false);
        last_frame_input.set_value_as_number((board_size / 2) as f64);
    }

    if state_params.spawn != u32::MAX
    {
        spawn_checkbox.set_checked(true);
        smooth_transform_checkbox.set_checked(state_params.smooth_transform);

        spawn_range.set_value_as_number(state_params.spawn as f64);
        spawn_input.set_value(&state_params.spawn.to_string());
    }
    else
    {
        spawn_checkbox.set_checked(false);
        smooth_transform_checkbox.set_checked(false);

        spawn_range.set_value_as_number(8 as f64);
        spawn_input.set_value("8");
    }
}

fn update_ui(run_state: RunState)
{
    let document = web_sys::window().unwrap().document().unwrap();

    let play_pause_button = document.get_element_by_id("button_play_pause").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    let stop_button       = document.get_element_by_id("button_stop_record").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    if run_state == RunState::Running || run_state == RunState::Recording
    {
        play_pause_button.set_text_content(Some("⏸️"));
    }
    else
    {
        play_pause_button.set_text_content(Some("▶️"));
    }

    if run_state == RunState::Stopped
    {
        stop_button.set_text_content(Some("⏺️"));
    }
    else
    {
        stop_button.set_text_content(Some("⏹️"));
    }

    let next_frame_button = document.get_element_by_id("button_next_frame").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    next_frame_button.set_disabled(run_state == RunState::Running || run_state == RunState::Recording || run_state == RunState::PausedRecording);

    let initial_board_select = document.get_element_by_id("initial_states").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
    initial_board_select.set_disabled(run_state != RunState::Stopped);

    let size_select = document.get_element_by_id("sizes").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
    size_select.set_disabled(run_state != RunState::Stopped);

    let spawn_checkbox = document.get_element_by_id("spawn_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    spawn_checkbox.set_disabled(run_state != RunState::Stopped);

    let spawn_decrement_button = document.get_element_by_id("decrement_spawn").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    spawn_decrement_button.set_disabled(run_state != RunState::Stopped || !spawn_checkbox.checked());

    let spawn_increment_button = document.get_element_by_id("increment_spawn").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    spawn_increment_button.set_disabled(run_state != RunState::Stopped || !spawn_checkbox.checked());

    let spawn_slider = document.get_element_by_id("spawn_range").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    spawn_slider.set_disabled(run_state != RunState::Stopped || !spawn_checkbox.checked());

    let smooth_transform_checkbox = document.get_element_by_id("smooth_transform_checkbox").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
    smooth_transform_checkbox.set_disabled(run_state != RunState::Stopped || !spawn_checkbox.checked());

    let upload_restriction_button = document.get_element_by_id("button_upload_restriction").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    upload_restriction_button.set_disabled(run_state != RunState::Stopped);

    let clear_restriction_button = document.get_element_by_id("button_clear_restriction").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    clear_restriction_button.set_disabled(run_state != RunState::Stopped);
}

fn update_next_frame_button_paused_recording(next_video_frame_available: bool)
{
    let document = web_sys::window().unwrap().document().unwrap();

    let next_frame_button = document.get_element_by_id("button_next_frame").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    next_frame_button.set_disabled(!next_video_frame_available);
}

fn update_last_frame_with_size(new_size: u32, app_state: &mut app_state::AppState)
{
    let document         = web_sys::window().unwrap().document().unwrap();
    let last_frame_input = document.get_element_by_id("last_frame_number").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let new_last_frame = new_size / 2;
    last_frame_input.set_value_as_number(new_last_frame as f64);

    if app_state.last_frame != u32::MAX
    {
        app_state.last_frame = new_last_frame;
    }
}