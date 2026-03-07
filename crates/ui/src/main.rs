mod app;
mod components;
mod pages;
mod routes;

use dioxus::prelude::*;

fn main() {
    tracing_subscriber::fmt::init();
    LaunchBuilder::new().launch(app::App);
}
