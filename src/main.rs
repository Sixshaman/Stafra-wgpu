#![cfg(not(target_arch = "wasm32"))]

pub mod app_state;
pub mod stafra_state;
pub mod video_record_state;
mod event_loop_native;

fn main()
{
    env_logger::init();
    futures::executor::block_on(event_loop_native::run_event_loop());
}