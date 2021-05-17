mod dummy_waker;
mod stafra_state;
mod app_state;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
#[cfg(target_arch = "wasm32")]
pub fn entry_point()
{
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("Error initiallizing logger");

    let app_state = app_state::AppState::new_web();
    wasm_bindgen_futures::spawn_local(app_state.run());
}