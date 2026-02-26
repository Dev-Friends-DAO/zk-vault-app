use dioxus::prelude::*;

use crate::pages::backup::Backup;
use crate::pages::dashboard::Dashboard;
use crate::pages::login::Login;
use crate::pages::register::Register;
use crate::pages::restore::Restore;
use crate::pages::settings::Settings;
use crate::pages::sources::Sources;
use crate::pages::verify::Verify;

#[derive(Routable, Clone, Debug, PartialEq)]
pub enum Route {
    #[route("/")]
    Dashboard {},

    #[route("/login")]
    Login {},

    #[route("/register")]
    Register {},

    #[route("/sources")]
    Sources {},

    #[route("/backup")]
    Backup {},

    #[route("/restore")]
    Restore {},

    #[route("/verify")]
    Verify {},

    #[route("/settings")]
    Settings {},
}
