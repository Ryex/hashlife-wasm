#![recursion_limit = "512"]

// #[cfg(feature = "wee_alloc")]
// #[global_allocator]
// static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;



#[macro_use]
mod utils;

mod app;
mod fps;
mod game;
pub mod universe;

#[cfg(not(feature = "no-wasm"))]
use wasm_bindgen::prelude::*;

#[cfg(not(feature = "no-wasm"))]
#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    utils::set_panic_hook();
    yew::start_app::<app::App>();

    log!("Starting!");

    Ok(())
}
