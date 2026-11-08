pub mod api;
pub mod format;
pub mod icons;
pub mod session;
pub mod storage;
pub mod styles;
pub mod views;
pub mod ws_client;

use dioxus::prelude::*;
use shared::dto::UserProfile;
use uuid::Uuid;

use crate::api::ApiClient;
use crate::icons::{Icon, IconKind};
use crate::session::{BackStack, Layer, LiveStatus, SessionExpired};
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

/// Aceita apenas tokens com formato de JWT (base64url separado por pontos)
fn is_plausible_token(token: &str) -> bool {
    (20..=8192).contains(&token.len())
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}

/// Extrai o access_token de um fragmento como "#access_token=...&expires_in=..."
fn extract_access_token(fragment: &str) -> Option<String> {
    fragment
        .trim_start_matches(['#', '?'])
        .split('&')
        .find_map(|pair| pair.strip_prefix("access_token="))
        .filter(|t| is_plausible_token(t))
        .map(str::to_string)
}

#[component]
fn App() -> Element {
    let initial_session = crate::storage::load_session_fs();
    let initial_token = initial_session.as_ref().map(|s| s.token.clone());
    let initial_user = initial_session.map(|s| s.user);

    let mut api_client = use_signal(|| ApiClient::new(None, initial_token.clone()));
    let mut current_user = use_signal(|| initial_user);
    let mut auth_token = use_signal(|| initial_token);
    let mut active_list_id = use_signal(|| None::<Uuid>);
    let mut error_msg = use_signal(|| None::<String>);
    let success_msg = use_signal(|| None::<String>);

    let session_expired = use_context_provider(|| SessionExpired(Signal::new(false)));
    let live_status = use_context_provider(|| LiveStatus(Signal::new(false)));
    let back = use_context_provider(BackStack::new);

    let mut do_logout = move || {
        back.reset();
        current_user.set(None);
        auth_token.set(None);
        active_list_id.set(None);
        api_client.set(ApiClient::new(None, None));
        crate::storage::clear_session();
    };

    let handle_logout = move |_| do_logout();

    let mut login_with = move |token: String, user: UserProfile| {
        crate::storage::save_session(&token, &user);
        let base_url = api_client.peek().base_url.clone();
        auth_token.set(Some(token.clone()));
        current_user.set(Some(user));
        api_client.set(ApiClient::new(Some(base_url), Some(token)));
    };

    // Qualquer tela que receber 401 dispara o logout por aqui
    use_effect(move || {
        let mut expired = session_expired.0;
        if *expired.read() {
            expired.set(false);
            if current_user.peek().is_some() {
                do_logout();
                error_msg.set(Some("Sua sessão expirou. Entre novamente.".to_string()));
            }
        }
    });

    // Botão/gesto "voltar" do Android: cada `popstate` fecha a camada do topo
    use_hook(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                window.addEventListener("popstate", function () { dioxus.send(true); });
                "#,
            );
            while eval.recv::<bool>().await.is_ok() {
                back.on_back();
            }
        });
    });

    // Lista aberta é uma camada: o "voltar" retorna ao painel
    use_effect(move || {
        if active_list_id.read().is_some() {
            back.open(Layer::List);
        } else {
            back.close(Layer::List);
        }
    });
    use_effect(move || {
        if let Some((Layer::List, _)) = *back.closed.read() {
            active_list_id.set(None);
        }
    });

    // Valida a sessão salva no arquivo assim que o app abre
    use_hook(move || {
        let client = api_client.peek().clone();
        if client.token.is_some() {
            spawn(async move {
                if let Err(err) = client.get_me().await {
                    if err.is_unauthorized() {
                        do_logout();
                    }
                }
            });
        }
    });

    // Restaura a sessão salva no localStorage (versão web) caso não exista arquivo
    use_hook(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                try {
                    let s = localStorage.getItem("app_compras_session");
                    dioxus.send(s || "");
                } catch(e) {
                    dioxus.send("");
                }
                "#,
            );
            let Ok(val) = eval.recv::<String>().await else { return };
            if val.trim().is_empty() || current_user.peek().is_some() {
                return;
            }
            let Ok(sess) = serde_json::from_str::<crate::storage::StoredSession>(&val) else { return };

            let base_url = api_client.peek().base_url.clone();
            let candidate = ApiClient::new(Some(base_url), Some(sess.token.clone()));
            match candidate.get_me().await {
                Ok(user) => login_with(sess.token, user),
                Err(err) if err.is_unauthorized() => crate::storage::clear_session(),
                // Sem internet: usa a sessão salva e valida depois
                Err(_) => login_with(sess.token, sess.user),
            }
        });
    });

    // Login com Google: o token chega pela URL (web) ou pelo deep link nativo (Android)
    use_hook(move || {
        spawn(async move {
            let mut eval = document::eval(
                r#"
                function sendHashIfPresent() {
                    let hash = window.location.hash || window.location.search || "";
                    if (hash.includes("access_token=")) {
                        dioxus.send(hash);
                        window.history.replaceState({}, document.title, window.location.pathname);
                    }
                }
                window.addEventListener("hashchange", sendHashIfPresent);
                window.onAppComprasToken = function(h) {
                    dioxus.send(String(h));
                };
                sendHashIfPresent();
                "#,
            );
            while let Ok(val) = eval.recv::<String>().await {
                let Some(access_token) = extract_access_token(&val) else { continue };
                // Deep links podem vir de qualquer site/app: nunca troca uma sessão ativa e
                // só aceita o token se o login com Google foi iniciado aqui há pouco
                if current_user.peek().is_some() || !crate::storage::take_oauth_pending().await {
                    continue;
                }
                let client = ApiClient::new(Some(api_client.peek().base_url.clone()), None);
                match client.google_auth(access_token).await {
                    Ok(res) => {
                        error_msg.set(None);
                        login_with(res.token, res.user);
                    }
                    Err(e) => {
                        error_msg.set(Some(format!("Falha na autenticação Google: {}", e)));
                    }
                }
            }
        });
    });

    let is_live = *live_status.0.read();

    rsx! {
        style { "{APP_CSS}" }

        div { class: "app-container",
            // Header / Navbar exibido quando autenticado
            if let Some(user) = current_user.read().as_ref() {
                header { class: "navbar",
                    div { class: "navbar-brand",
                        div { class: "brand-badge", Icon { kind: IconKind::Cart, size: 16 } }
                        span { "App Compras" }
                        if is_live {
                            div { class: "live-indicator",
                                span { class: "live-dot" }
                                span { "Ao vivo" }
                            }
                        }
                    }

                    div { class: "nav-actions",
                        div { class: "user-chip",
                            div { class: "user-avatar-circle", "aria-hidden": "true",
                                "{user.name.chars().next().unwrap_or('U').to_uppercase()}"
                            }
                            span { "{user.name}" }
                        }
                        button {
                            class: "btn-logout",
                            onclick: handle_logout,
                            "Sair"
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
                    key: "{list_id}",
                    api_client,
                    current_user,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_only_valid_tokens() {
        let jwt = "eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIn0.abc-def_123";
        assert_eq!(
            extract_access_token(&format!("#access_token={}&expires_in=3600", jwt)).as_deref(),
            Some(jwt)
        );
        assert_eq!(extract_access_token("#access_token=';alert(1)//"), None);
        assert_eq!(extract_access_token("#foo=bar"), None);
    }
}
