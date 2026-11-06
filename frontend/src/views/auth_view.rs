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
            }
        }
    }
}
