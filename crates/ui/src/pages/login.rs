use dioxus::prelude::*;

use crate::routes::Route;

#[component]
pub fn Login() -> Element {
    let mut passphrase = use_signal(String::new);
    let error_msg = use_signal(|| None::<String>);

    let on_submit = move |evt: FormEvent| {
        evt.prevent_default();
        // TODO: OPAQUE login flow
        tracing::info!("Login attempt");
    };

    rsx! {
        div { class: "min-h-screen flex items-center justify-center",
            div { class: "w-full max-w-md",
                // Logo / Title
                div { class: "text-center mb-8",
                    h1 { class: "text-3xl font-bold text-white", "zk-vault" }
                    p { class: "text-gray-400 mt-2", "Post-Quantum Secure Backup" }
                }

                // Login form
                form {
                    onsubmit: on_submit,
                    class: "bg-gray-800 rounded-lg p-6 border border-gray-700 space-y-4",

                    h2 { class: "text-xl font-semibold text-white", "Sign In" }

                    if let Some(err) = error_msg() {
                        div { class: "bg-red-900/50 border border-red-700 text-red-300 px-4 py-2 rounded",
                            "{err}"
                        }
                    }

                    div {
                        label { class: "block text-sm text-gray-300 mb-1", r#for: "passphrase",
                            "Passphrase"
                        }
                        input {
                            id: "passphrase",
                            r#type: "password",
                            class: "w-full px-3 py-2 bg-gray-900 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:border-indigo-500",
                            placeholder: "Enter your passphrase",
                            value: "{passphrase}",
                            oninput: move |evt| passphrase.set(evt.value()),
                        }
                    }

                    button {
                        r#type: "submit",
                        class: "w-full py-2 bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium transition-colors",
                        "Sign In"
                    }

                    div { class: "text-center text-sm text-gray-400",
                        "Don't have an account? "
                        Link {
                            to: Route::Register {},
                            class: "text-indigo-400 hover:text-indigo-300",
                            "Create one"
                        }
                    }
                }
            }
        }
    }
}
