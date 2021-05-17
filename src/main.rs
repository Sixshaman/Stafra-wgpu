mod dummy_waker;
mod stafra_state;
mod app_state;

#[cfg(not(target_arch = "wasm32"))]
fn main()
{
    env_logger::init();

    let app_state = app_state::AppState::new_native();
    futures::executor::block_on(app_state.run());
}