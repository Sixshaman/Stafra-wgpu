#![cfg(target_arch = "wasm32")]

use std::task::{RawWakerVTable, RawWaker, Waker};

pub fn dummy_waker() -> Waker
{
    unsafe {Waker::from_raw(dummy_waker_raw())}
}

fn dummy_waker_raw() -> RawWaker
{
    RawWaker::new(std::ptr::null(), &DUMMY_WAKER_VTABLE)
}

static DUMMY_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new
(
    |_| {dummy_waker_raw()},
    |_| {},
    |_| {},
    |_| {}
);