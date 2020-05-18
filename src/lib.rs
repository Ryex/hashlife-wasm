#[macro_use]
mod utils;

mod app;

use wasm_bindgen::prelude::*;
use yew::prelude::*;

#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    yew::start_app::<app::App>();

    log!("Starting!");
    Ok(())
}
