#![cfg(not(target_arch = "wasm32"))]

mod app_state;
mod stafra_state;
mod event_loop_native;

fn main()
{
    env_logger::init();
    futures::executor::block_on(event_loop_native::run_event_loop());
}