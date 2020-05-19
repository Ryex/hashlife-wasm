#![recursion_limit = "512"]

#[macro_use]
mod utils;

mod app;
mod fps;
pub mod gameoflife;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    utils::set_panic_hook();
    yew::start_app::<app::App>();

    log!("Starting!");

    Ok(())
}
