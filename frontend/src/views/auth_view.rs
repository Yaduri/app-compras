use dioxus::prelude::*;
use shared::dto::{LoginRequest, RegisterRequest, UserProfile};
use uuid::Uuid;

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

    let handle_demo_login = move |_| {
        let client = api_client.read().clone();
        is_loading.set(true);
        error_msg.set(None);

        spawn(async move {
            // Tenta o endpoint oficial /api/auth/demo
            let demo_res = client.demo().await;
            is_loading.set(false);

            match demo_res {
                Ok(auth_res) => {
                    auth_token.set(Some(auth_res.token.clone()));
                    current_user.set(Some(auth_res.user));
                    api_client.set(ApiClient::new(Some(client.base_url), Some(auth_res.token)));
                }
                Err(_) => {
                    // Fallback imediato: sessão demo local
                    let demo_user = UserProfile {
                        id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                        email: "demo@mercado.com".to_string(),
                        name: "Usuário Demo".to_string(),
                        avatar_url: None,
                    };
                    auth_token.set(Some("demo-local-token".to_string()));
                    current_user.set(Some(demo_user));
                }
            }
        });
    };

    rsx! {
        div { class: "main-content", style: "display: flex; flex-direction: column; justify-content: center; max-width: 440px; margin: 30px auto; min-height: 85vh;",
            // Hero / Branding
            div { style: "text-align: center; margin-bottom: 28px;",
                div { style: "width: 72px; height: 72px; margin: 0 auto 16px auto; background: linear-gradient(135deg, #d1fae5 0%, #ecfdf5 100%); border-radius: 22px; display: flex; align-items: center; justify-content: center; font-size: 2.2rem; box-shadow: 0 10px 25px -4px rgba(16, 185, 129, 0.28); border: 1.5px solid #a7f3d0;",
                    "🛒"
                }
                h2 { style: "font-size: 1.85rem; color: #0f172a; font-weight: 800; letter-spacing: -0.03em;",
                    "App Compras"
                }
                p { style: "color: #64748b; font-size: 0.95rem; margin-top: 6px; max-width: 330px; margin-left: auto; margin-right: auto; line-height: 1.4;",
                    "Lista de supermercado colaborativa em tempo real para sua casa e família"
                }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error",
                    span { style: "font-size: 1.1rem;", "⚠️" }
                    span { "{err}" }
                }
            }

            div { class: "card",
                // Seletor iOS-style de Abas
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
                    style: "width: 100%; margin-top: 6px;",
                    disabled: *is_loading.read(),
                    onclick: handle_submit,
                    if *is_loading.read() {
                        "Carregando..."
                    } else if *is_register.read() {
                        "Concluir Cadastro"
                    } else {
                        "Entrar no App"
                    }
                }

                div { style: "text-align: center; margin: 20px 0 14px 0; color: #94a3b8; font-size: 0.78rem; font-weight: 700; letter-spacing: 0.06em;",
                    "— OU EXPERIMENTE SEM CADASTRO —"
                }

                button {
                    class: "btn",
                    style: "width: 100%; background: linear-gradient(135deg, #f0fdf4 0%, #ecfdf5 100%); border: 1.5px solid #a7f3d0; color: #065f46; font-weight: 700;",
                    disabled: *is_loading.read(),
                    onclick: handle_demo_login,
                    span { style: "font-size: 1.1rem;", "⚡" }
                    span { "Acesso Rápido Demo (1 Clique)" }
                }
            }
        }
    }
}
