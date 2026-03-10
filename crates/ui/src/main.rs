mod app;
mod components;
mod pages;
mod routes;

use dioxus::prelude::*;

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    LaunchBuilder::new().launch(app::App);
}
