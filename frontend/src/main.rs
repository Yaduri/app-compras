pub mod api;
pub mod styles;
pub mod views;
pub mod ws_client;

use dioxus::prelude::*;
use shared::dto::UserProfile;
use uuid::Uuid;

use crate::api::ApiClient;
use crate::styles::APP_CSS;
use crate::views::{AuthView, DashboardView, ListDetailView};

fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    dotenvy::dotenv().ok();

    #[cfg(target_os = "android")]
    {
        dioxus::LaunchBuilder::mobile().launch(App);
    }

    #[cfg(target_os = "ios")]
    {
        dioxus::LaunchBuilder::mobile().launch(App);
    }

    #[cfg(all(feature = "desktop", not(any(target_os = "android", target_os = "ios", target_arch = "wasm32"))))]
    {
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};
        let cfg = Config::new()
            .with_window(
                WindowBuilder::new()
                    .with_title("App Compras")
                    .with_inner_size(LogicalSize::new(500.0, 850.0))
                    .with_visible(true),
            );

        dioxus::LaunchBuilder::desktop()
            .with_cfg(cfg)
            .launch(App);
    }

    #[cfg(all(not(feature = "desktop"), not(target_os = "android"), not(target_os = "ios")))]
    {
        dioxus::launch(App);
    }
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

    // Auto-login se houver callback do Google / Supabase na URL
    use_effect(move || {
        let client = api_client.read().clone();
        spawn(async move {
            let mut eval = document::eval(
                r#"
                let hash = window.location.hash || window.location.search || "";
                dioxus.send(hash);
                "#,
            );
            if let Ok(val) = eval.recv::<String>().await {
                if val.contains("access_token=") {
                    let token_part = val.split("access_token=").nth(1).unwrap_or("");
                    let access_token = token_part.split('&').next().unwrap_or("");
                    if !access_token.is_empty() {
                        if let Ok(res) = client.google_auth(access_token.to_string()).await {
                            auth_token.set(Some(res.token.clone()));
                            current_user.set(Some(res.user));
                            api_client.set(ApiClient::new(Some(client.base_url), Some(res.token)));
                            let _ = document::eval("window.history.replaceState({}, document.title, window.location.pathname);");
                        }
                    }
                }
            }
        });
    });

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
                                class: "btn-logout",
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
