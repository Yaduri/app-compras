use dioxus::prelude::*;
use shared::dto::{LoginRequest, RegisterRequest, UserProfile};

use crate::api::ApiClient;

#[component]
pub fn AuthView(
    api_client: Signal<ApiClient>,
    current_user: Signal<Option<UserProfile>>,
    auth_token: Signal<Option<String>>,
    error_msg: Signal<Option<String>>,
) -> Element {
    let mut is_register = use_signal(|| false);
    let mut name_input = use_signal(String::new);
    let mut email_input = use_signal(String::new);
    let mut password_input = use_signal(String::new);
    let mut is_loading = use_signal(|| false);

    let handle_submit = move |_| {
        let is_reg = *is_register.read();
        let name = name_input.read().clone();
        let email = email_input.read().clone();
        let password = password_input.read().clone();
        let client = api_client.read().clone();

        if email.trim().is_empty() || password.trim().is_empty() {
            error_msg.set(Some("Por favor, preencha todos os campos obrigatórios".to_string()));
            return;
        }

        if is_reg && name.trim().is_empty() {
            error_msg.set(Some("Por favor, informe seu nome".to_string()));
            return;
        }

        is_loading.set(true);
        error_msg.set(None);

        spawn(async move {
            let result = if is_reg {
                client
                    .register(RegisterRequest {
                        name: name.clone(),
                        email: email.clone(),
                        password: password.clone(),
                    })
                    .await
            } else {
                client
                    .login(LoginRequest {
                        email: email.clone(),
                        password: password.clone(),
                    })
                    .await
            };

            is_loading.set(false);

            match result {
                Ok(auth_res) => {
                    crate::storage::save_session(&auth_res.token, &auth_res.user);
                    auth_token.set(Some(auth_res.token.clone()));
                    current_user.set(Some(auth_res.user));
                    api_client.set(ApiClient::new(Some(client.base_url), Some(auth_res.token)));
                }
                Err(err) => {
                    error_msg.set(Some(err));
                }
            }
        });
    };

    let handle_google_login = move |_| {
        let auth_url = "https://xfzcsdtsjundbhhxqwjm.supabase.co/auth/v1/authorize?provider=google&redirect_to=https://app-compras-production.up.railway.app";
        let _ = document::eval(&format!("window.location.href = '{}';", auth_url));
    };

    rsx! {
        div { class: "main-content", style: "display: flex; flex-direction: column; justify-content: center; max-width: 400px; margin: 40px auto; min-height: 80vh;",
            // Hero / Branding Minimalista (Linear / Things 3)
            div { style: "text-align: center; margin-bottom: 32px;",
                div { style: "width: 56px; height: 56px; margin: 0 auto 16px auto; background: var(--carbon-dark); border-radius: 16px; display: flex; align-items: center; justify-content: center; color: #ffffff; font-size: 1.4rem; font-weight: 800; box-shadow: 0 4px 14px rgba(15, 23, 42, 0.15);",
                    "✓"
                }
                h2 { style: "font-size: 1.6rem; color: var(--text-primary); font-weight: 800; letter-spacing: -0.03em;",
                    "App Compras"
                }
                p { style: "color: var(--text-secondary); font-size: 0.88rem; margin-top: 4px; line-height: 1.4;",
                    "Listas compartilhadas em tempo real"
                }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error",
                    span { "{err}" }
                }
            }

            div { class: "card",
                // Seletor Minimalista de Abas
                div { class: "segmented-control",
                    button {
                        class: if !*is_register.read() { "segmented-btn active" } else { "segmented-btn" },
                        onclick: move |_| {
                            is_register.set(false);
                            error_msg.set(None);
                        },
                        "Entrar"
                    }
                    button {
                        class: if *is_register.read() { "segmented-btn active" } else { "segmented-btn" },
                        onclick: move |_| {
                            is_register.set(true);
                            error_msg.set(None);
                        },
                        "Criar Conta"
                    }
                }

                if *is_register.read() {
                    div { class: "form-group",
                        label { class: "form-label", "Seu Nome Completo" }
                        input {
                            class: "form-input",
                            placeholder: "Ex: Maria Silva",
                            value: "{name_input}",
                            oninput: move |e| name_input.set(e.value())
                        }
                    }
                }

                div { class: "form-group",
                    label { class: "form-label", "E-mail" }
                    input {
                        class: "form-input",
                        r#type: "email",
                        placeholder: "seu@email.com",
                        value: "{email_input}",
                        oninput: move |e| email_input.set(e.value())
                    }
                }

                div { class: "form-group",
                    label { class: "form-label", "Senha" }
                    input {
                        class: "form-input",
                        r#type: "password",
                        placeholder: "Mínimo 6 caracteres",
                        value: "{password_input}",
                        oninput: move |e| password_input.set(e.value())
                    }
                }

                button {
                    class: "btn btn-primary",
                    style: "width: 100%; margin-top: 8px;",
                    disabled: *is_loading.read(),
                    onclick: handle_submit,
                    if *is_loading.read() {
                        "Carregando..."
                    } else if *is_register.read() {
                        "Concluir Cadastro"
                    } else {
                        "Entrar"
                    }
                }

                div { style: "text-align: center; margin: 18px 0 12px 0; color: var(--text-muted); font-size: 0.72rem; font-weight: 700; letter-spacing: 0.08em;",
                    "— OU —"
                }

                button {
                    class: "btn-google",
                    disabled: *is_loading.read(),
                    onclick: handle_google_login,
                    svg {
                        width: "18",
                        height: "18",
                        view_box: "0 0 24 24",
                        style: "margin-right: 8px; flex-shrink: 0;",
                        path {
                            fill: "#4285F4",
                            d: "M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
                        }
                        path {
                            fill: "#34A853",
                            d: "M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
                        }
                        path {
                            fill: "#FBBC05",
                            d: "M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.06H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.94l2.85-2.22.81-.63z"
                        }
                        path {
                            fill: "#EA4335",
                            d: "M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.06l3.66 2.84c.87-2.6 3.3-4.52 6.16-4.52z"
                        }
                    }
                    span { "Continuar com o Google" }
                }
            }
        }
    }
}
