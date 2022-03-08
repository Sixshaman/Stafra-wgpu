#![cfg(target_arch = "wasm32")]

mod dummy_waker;
mod app_state;
mod stafra_state;
mod event_loop_web;

use wasm_bindgen::prelude::*;
use console_log;
use console_error_panic_hook;

#[wasm_bindgen(start)]
pub fn entry_point()
{
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("Error initiallizing logger");

    wasm_bindgen_futures::spawn_local(event_loop_web::run_event_loop());
}