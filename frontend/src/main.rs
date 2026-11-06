pub mod api;
pub mod styles;
pub mod views;
pub mod ws_client;

use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
use dioxus::prelude::*;
use shared::dto::UserProfile;
use uuid::Uuid;

use crate::api::ApiClient;
use crate::styles::APP_CSS;
use crate::views::{AuthView, DashboardView, ListDetailView};

fn main() {
    let cfg = Config::new()
        .with_window(
            WindowBuilder::new()
                .with_title("App Compras Colaborativo")
                .with_inner_size(LogicalSize::new(500.0, 850.0))
                .with_visible(true),
        );

    LaunchBuilder::desktop().with_cfg(cfg).launch(App);
}

#[component]
fn App() -> Element {
    let mut api_client = use_signal(|| ApiClient::new(None, None));
    let mut current_user = use_signal(|| None::<UserProfile>);
    let mut auth_token = use_signal(|| None::<String>);
    let mut active_list_id = use_signal(|| None::<Uuid>);
    let error_msg = use_signal(|| None::<String>);
    let success_msg = use_signal(|| None::<String>);

    let handle_logout = move |_| {
        current_user.set(None);
        auth_token.set(None);
        active_list_id.set(None);
        api_client.set(ApiClient::new(None, None));
    };

    rsx! {
        style { "{APP_CSS}" }

        div { class: "app-container",
            // Header / Navbar exibido quando autenticado
            if current_user.read().is_some() {
                header { class: "navbar",
                    h1 {
                        div { class: "brand-badge", "🛒" }
                        span { "App Compras" }
                        div { class: "live-indicator",
                            span { class: "live-dot" }
                            span { "Ao vivo" }
                        }
                    }

                    if let Some(user) = current_user.read().as_ref() {
                        div { class: "nav-actions",
                            div { class: "user-chip",
                                div { class: "user-avatar-circle", "{user.name.chars().next().unwrap_or('U')}" }
                                span { "{user.name}" }
                            }
                            button {
                                class: "btn-icon",
                                title: "Desconectar",
                                onclick: handle_logout,
                                "Sair"
                            }
                        }
                    }
                }
            }

            // Roteamento Reativo de Telas
            if current_user.read().is_none() {
                AuthView {
                    api_client,
                    current_user,
                    auth_token,
                    error_msg,
                }
            } else if let Some(list_id) = *active_list_id.read() {
                ListDetailView {
                    api_client,
                    list_id,
                    active_list_id,
                    error_msg,
                    success_msg,
                }
            } else {
                DashboardView {
                    api_client,
                    current_user,
                    active_list_id,
                    error_msg,
                    success_msg,
                }
            }
        }
    }
}
