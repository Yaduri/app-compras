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

    let handle_demo_login = move |_| {
        let client = api_client.read().clone();
        is_loading.set(true);
        error_msg.set(None);

        spawn(async move {
            // Tenta logar como usuário demo ou cadastra-o se não existir
            let demo_email = "demo@mercado.com".to_string();
            let demo_pass = "demo123456".to_string();

            let login_res = client
                .login(LoginRequest {
                    email: demo_email.clone(),
                    password: demo_pass.clone(),
                })
                .await;

            let final_res = match login_res {
                Ok(res) => Ok(res),
                Err(_) => {
                    // Cadastra a conta demo se ainda não existir
                    client
                        .register(RegisterRequest {
                            name: "Usuário Demo".to_string(),
                            email: demo_email,
                            password: demo_pass,
                        })
                        .await
                }
            };

            is_loading.set(false);

            match final_res {
                Ok(auth_res) => {
                    auth_token.set(Some(auth_res.token.clone()));
                    current_user.set(Some(auth_res.user));
                    api_client.set(ApiClient::new(Some(client.base_url), Some(auth_res.token)));
                }
                Err(err) => {
                    error_msg.set(Some(format!("Não foi possível entrar no modo Demo: {}", err)));
                }
            }
        });
    };

    rsx! {
        div { class: "main-content", style: "display: flex; flex-direction: column; justify-content: center; max-width: 420px; margin: 40px auto;",
            div { style: "text-align: center; margin-bottom: 28px;",
                div { style: "font-size: 3rem; margin-bottom: 8px;", "🛒" }
                h2 { style: "font-size: 1.6rem; color: #0284c7; font-weight: 800;", "App Compras" }
                p { style: "color: #64748b; font-size: 0.95rem; margin-top: 4px;",
                    "Lista de compras colaborativa em tempo real para casais e famílias"
                }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error", "{err}" }
            }

            div { class: "card",
                div { style: "display: flex; border-bottom: 1px solid #e2e8f0; margin-bottom: 20px;",
                    button {
                        class: if !*is_register.read() { "btn" } else { "btn btn-secondary" },
                        style: "flex: 1; border-radius: 8px 8px 0 0; border: none; font-weight: 700;",
                        onclick: move |_| {
                            is_register.set(false);
                            error_msg.set(None);
                        },
                        "Entrar"
                    }
                    button {
                        class: if *is_register.read() { "btn" } else { "btn btn-secondary" },
                        style: "flex: 1; border-radius: 8px 8px 0 0; border: none; font-weight: 700;",
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
                    label { class: "form-label", "Senha (mínimo 6 caracteres)" }
                    input {
                        class: "form-input",
                        r#type: "password",
                        placeholder: "••••••••",
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
                        "Entrar no App"
                    }
                }

                div { style: "text-align: center; margin: 18px 0; color: #94a3b8; font-size: 0.85rem;",
                    "— ou experimente agora —"
                }

                button {
                    class: "btn btn-secondary",
                    style: "width: 100%; border-color: #38bdf8; color: #0284c7; background-color: #f0f9ff;",
                    disabled: *is_loading.read(),
                    onclick: handle_demo_login,
                    "⚡ Acesso Rápido de Teste (Demo)"
                }
            }
        }
    }
}
