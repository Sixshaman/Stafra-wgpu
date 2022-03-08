#![cfg(target_arch = "wasm32")]

use
{
    wasm_bindgen::closure::Closure,
    wasm_bindgen::{JsCast, Clamped},
    std::rc::Rc,
    std::cell::RefCell
};

use super::stafra_state;
use super::app_state;
use crate::app_state::RunState;

pub async fn run_event_loop()
{
    //Obtaining document elements
    let window   = web_sys::window().unwrap();
    let document = web_sys::window().unwrap().document().unwrap();

    let main_canvas       = document.get_element_by_id("stafra_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let click_rule_canvas = document.get_element_by_id("click_rule_canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

    let save_png_button = document.get_element_by_id("button_save_png").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let play_pause_button = document.get_element_by_id("button_play_pause").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    let stop_button       = document.get_element_by_id("button_stop").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    let next_frame_button = document.get_element_by_id("button_next_frame").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();

    let initial_state_select       = document.get_element_by_id("initial_states").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
    let initial_state_upload_input = document.get_element_by_id("board_input").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();

    let board_upload_image = web_sys::HtmlImageElement::new().unwrap();
    let board_file_reader  = web_sys::FileReader::new().unwrap();


    //Initializing the state
    let initial_width  = 1023;
    let initial_height = 1023;

    let app_state_rc    = Rc::new(RefCell::new(app_state::AppState::new()));
    let stafra_state_rc = Rc::new(RefCell::new(stafra_state::StafraState::new_web(&main_canvas, &click_rule_canvas, initial_width, initial_height).await));

    let mut app_state   = app_state_rc.borrow_mut();
    let mut stafra_state = stafra_state_rc.borrow_mut();

    app_state.run_state = RunState::Running;

    stafra_state.reset_board_standard(stafra_state::StandardResetBoardType::Corners);
    stafra_state.reset_click_rule();

    //Canvas resize handler
    let stafra_state_clone_for_resize = stafra_state_rc.clone();
    main_canvas.set_onresize(Some(Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut stafra_state = stafra_state_clone_for_resize.borrow_mut();

        let canvas = event.target().unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        stafra_state.resize(canvas.client_width() as u32, canvas.client_height() as u32);
    }) as Box<dyn Fn(web_sys::Event)>).as_ref().unchecked_ref()));

    //Save png handler
    let stafra_state_clone_for_save_png = stafra_state_rc.clone();
    save_png_button.set_onclick(Some(Closure::wrap(Box::new(move ||
    {
        let mut stafra_state = stafra_state_clone_for_save_png.borrow_mut();
        stafra_state.post_png_data_request();
    }) as Box<dyn Fn()>).as_ref().unchecked_ref()));

    //Play/pause handler
    let app_state_clone_for_play_pause = app_state_rc.clone();
    play_pause_button.set_onclick(Some(Closure::wrap(Box::new(move ||
    {
        let mut app_state = app_state_clone_for_play_pause.borrow_mut();

        app_state.run_state = if app_state.run_state == RunState::Running {RunState::Paused} else {RunState::Running};
        update_ui(&app_state.run_state);
    }) as Box<dyn Fn()>).as_ref().unchecked_ref()));

    //Stop handler
    let app_state_clone_for_stop = app_state_rc.clone();
    let stafra_state_clone_for_stop = stafra_state_rc.clone();
    stop_button.set_onclick(Some(Closure::wrap(Box::new(move ||
    {
        let mut app_state  = app_state_clone_for_stop.borrow_mut();
        let mut stafra_state = stafra_state_clone_for_stop.borrow_mut();

        app_state.run_state = RunState::Stopped;
        stafra_state.reset_board_unchanged();

        update_ui(&app_state.run_state);
    }) as Box<dyn Fn()>).as_ref().unchecked_ref()));

    //Next frame handler
    let app_state_clone_for_next_frame = app_state_rc.clone();
    let stafra_state_clone_for_next_frame = stafra_state_rc.clone();
    next_frame_button.set_onclick(Some(Closure::wrap(Box::new(move ||
    {
        let app_state = app_state_clone_for_next_frame.borrow();
        let mut stafra_state = stafra_state_clone_for_next_frame.borrow_mut();

        if app_state.run_state != RunState::Running
        {
            stafra_state.update();
        }
    }) as Box<dyn Fn()>).as_ref().unchecked_ref()));

    //Board upload handler
    let app_state_clone_for_board_upload = app_state_rc.clone();
    let stafra_state_clone_for_board_upload = stafra_state_rc.clone();
    board_upload_image.set_onload(Some(Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut app_state  = app_state_clone_for_board_upload.borrow_mut();
        let mut stafra_state = stafra_state_clone_for_board_upload.borrow_mut();

        let board_image = event.target().unwrap().dyn_into::<web_sys::HtmlImageElement>().unwrap();

        let document       = web_sys::window().unwrap().document().unwrap();
        let canvas_board   = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let canvas_context = canvas_board.get_context("2d").unwrap().unwrap().dyn_into::<web_sys::CanvasRenderingContext2d>().unwrap();

        canvas_board.set_width(board_image.width());
        canvas_board.set_height(board_image.height());

        canvas_context.draw_image_with_html_image_element(&board_image, 0.0, 0.0).expect("Draw image error!");
        let image_data = canvas_context.get_image_data(0.0, 0.0, board_image.width() as f64, board_image.height() as f64).unwrap();

        stafra_state.reset_board_custom(image_data.data().to_vec(), image_data.width(), image_data.height());
        app_state.run_state = RunState::Running;

        canvas_board.remove();
        update_ui(&app_state.run_state);
    }) as Box<dyn Fn(web_sys::Event)>).as_ref().unchecked_ref()));

    //Board file read handler
    board_file_reader.set_onload(Some(Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let file_reader      = event.target().unwrap().dyn_into::<web_sys::FileReader>().unwrap();
        let file_read_result = file_reader.result().unwrap();

        let file_data = file_read_result.as_string().unwrap();
        board_upload_image.set_src(&file_data);
    }) as Box<dyn Fn(web_sys::Event)>).as_ref().unchecked_ref()));

    //Custom initial state upload handler
    initial_state_upload_input.set_onchange(Some(Closure::wrap(Box::new(move |event: web_sys::Event|
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
    }) as Box<dyn Fn(web_sys::Event)>).as_ref().unchecked_ref()));

    //Reset board
    let app_state_clone_for_initial_state_select = app_state_rc.clone();
    let stafra_state_clone_for_initial_state_select = stafra_state_rc.clone();
    initial_state_select.set_onchange(Some(Closure::wrap(Box::new(move |event: web_sys::Event|
    {
        let mut app_state  = app_state_clone_for_initial_state_select.borrow_mut();
        let mut stafra_state = stafra_state_clone_for_initial_state_select.borrow_mut();

        let board_reset_select = event.target().unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
        match board_reset_select.value().as_str()
        {
            "initial_state_corners" =>
            {
                stafra_state.reset_board_standard(stafra_state::StandardResetBoardType::Corners);
                app_state.run_state = RunState::Running;
            },

            "initial_state_sides" =>
            {
                stafra_state.reset_board_standard(stafra_state::StandardResetBoardType::Edges);
                app_state.run_state = RunState::Running;
            },

            "initial_state_center" =>
            {
                stafra_state.reset_board_standard(stafra_state::StandardResetBoardType::Center);
                app_state.run_state = RunState::Running;
            },

            "initial_state_custom" =>
            {
                let document = web_sys::window().unwrap().document().unwrap();
                let initial_state_upload_input = document.get_element_by_id("board_input").unwrap().dyn_into::<web_sys::HtmlInputElement>().unwrap();
                initial_state_upload_input.click();
            },

            _ => {}
        }

        update_ui(&app_state.run_state);
    }) as Box<dyn FnMut(web_sys::Event)>).as_ref().unchecked_ref()));


    //Refresh handler
    let app_state_clone_for_refresh = app_state_rc.clone();
    let stafra_state_clone_for_refresh = stafra_state_rc.clone();
    let refresh_function: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let refresh_function_copy = refresh_function.clone();
    *refresh_function_copy.borrow_mut() = Some(Closure::wrap(Box::new(move ||
    {
        let app_state = app_state_clone_for_refresh.borrow();
        let mut stafra_state = stafra_state_clone_for_refresh.borrow_mut();

        let window = web_sys::window().unwrap();

        //Update state
        if app_state.run_state == RunState::Running
        {
            stafra_state.update();
        }

        //Poll PNG save request
        match stafra_state.check_png_data_request()
        {
            Ok((mut image_array, width, height, row_pitch)) =>
            {
                let image_data = web_sys::ImageData::new_with_u8_clamped_array(Clamped(image_array.as_mut_slice()), row_pitch).unwrap();
                save_image_data(image_data, width, height);
            },

            Err(_) =>
            {
            }
        }

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

    update_ui(&app_state.run_state);
    window.request_animation_frame(refresh_function_copy.borrow().as_ref().unwrap().as_ref().unchecked_ref()).expect("Request animation frame error!");
}

fn save_image_data(image_data: web_sys::ImageData, width: u32, height: u32)
{
    let document = web_sys::window().unwrap().document().unwrap();

    let canvas = document.create_element("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    canvas.set_width(width);
    canvas.set_height(height);

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

fn update_ui(run_state: &RunState)
{
    let document = web_sys::window().unwrap().document().unwrap();

    let play_pause_button = document.get_element_by_id("button_play_pause").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    if run_state == &RunState::Running
    {
        play_pause_button.set_text_content(Some("⏸️"));
    }
    else
    {
        play_pause_button.set_text_content(Some("▶️"));
    }

    let next_frame_button = document.get_element_by_id("button_next_frame").unwrap().dyn_into::<web_sys::HtmlButtonElement>().unwrap();
    next_frame_button.set_disabled(run_state == &RunState::Running);

    let initial_board_select = document.get_element_by_id("initial_states").unwrap().dyn_into::<web_sys::HtmlSelectElement>().unwrap();
    initial_board_select.set_disabled(run_state != &RunState::Stopped);
}