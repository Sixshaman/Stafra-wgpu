#![cfg(target_arch = "wasm32")]

pub mod dummy_waker;
pub mod app_state;
pub mod stafra_state;
pub mod stafra_static_state;
pub mod stafra_static_state_bindings;
pub mod stafra_board_state_bindings;
pub mod stafra_initial_state_bindings;
pub mod video_record_state;
mod event_loop_web;

use wasm_bindgen::prelude::*;
use console_log;
use console_error_panic_hook;

#[wasm_bindgen(start)]
pub fn entry_point()
{
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("Error initializing logger");

    wasm_bindgen_futures::spawn_local(event_loop_web::run_event_loop());
}