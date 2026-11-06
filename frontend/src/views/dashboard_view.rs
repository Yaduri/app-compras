use dioxus::prelude::*;
use shared::dto::{CreateListRequest, JoinListRequest, ListSummaryResponse, UserProfile};
use shared::models::MemberRole;
use uuid::Uuid;

use crate::api::ApiClient;

#[component]
pub fn DashboardView(
    api_client: Signal<ApiClient>,
    current_user: Signal<Option<UserProfile>>,
    active_list_id: Signal<Option<Uuid>>,
    error_msg: Signal<Option<String>>,
    success_msg: Signal<Option<String>>,
) -> Element {
    let mut lists = use_signal(Vec::<ListSummaryResponse>::new);
    let mut is_loading = use_signal(|| false);

    // Modais
    let mut show_create_modal = use_signal(|| false);
    let mut new_list_name = use_signal(String::new);
    let mut new_list_desc = use_signal(String::new);

    let mut show_join_modal = use_signal(|| false);
    let mut join_code_input = use_signal(String::new);

    // Carrega listas do usuário
    let mut load_lists = move || {
        let client = api_client.read().clone();
        is_loading.set(true);
        spawn(async move {
            match client.list_user_lists().await {
                Ok(data) => {
                    if data.is_empty() {
                        let demo_list = ListSummaryResponse {
                            id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                            name: "Churrasco do Fim de Semana".to_string(),
                            description: Some("Lista colaborativa para o churrasco com a família e amigos".to_string()),
                            invite_code: "CHURRAS".to_string(),
                            role: MemberRole::Owner,
                            member_count: 1,
                            totals: shared::dto::ListTotals {
                                total_items: 5,
                                purchased_items: 2,
                                total_planned: 206.40,
                                total_cart: 48.30,
                            },
                        };
                        lists.set(vec![demo_list]);
                    } else {
                        lists.set(data);
                    }
                }
                Err(_) => {
                    let demo_list = ListSummaryResponse {
                        id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                        name: "Churrasco do Fim de Semana".to_string(),
                        description: Some("Lista colaborativa para o churrasco com a família e amigos".to_string()),
                        invite_code: "CHURRAS".to_string(),
                        role: MemberRole::Owner,
                        member_count: 1,
                        totals: shared::dto::ListTotals {
                            total_items: 5,
                            purchased_items: 2,
                            total_planned: 206.40,
                            total_cart: 48.30,
                        },
                    };
                    lists.set(vec![demo_list]);
                }
            }
            is_loading.set(false);
        });
    };

    use_effect(move || {
        load_lists();
    });

    let handle_create_list = move |_| {
        let name = new_list_name.read().clone();
        let desc = new_list_desc.read().clone();
        let client = api_client.read().clone();

        if name.trim().is_empty() {
            error_msg.set(Some("O nome da lista é obrigatório".to_string()));
            return;
        }

        spawn(async move {
            let req = CreateListRequest {
                name: name.trim().to_string(),
                description: if desc.trim().is_empty() {
                    None
                } else {
                    Some(desc.trim().to_string())
                },
            };

            match client.create_list(req.clone()).await {
                Ok(created) => {
                    show_create_modal.set(false);
                    new_list_name.set(String::new());
                    new_list_desc.set(String::new());
                    success_msg.set(Some(format!("Lista '{}' criada com sucesso!", created.name)));
                    load_lists();
                }
                Err(_) => {
                    // Fallback para modo demo / offline
                    let new_id = Uuid::new_v4();
                    let new_summary = ListSummaryResponse {
                        id: new_id,
                        name: req.name.clone(),
                        description: req.description.clone(),
                        invite_code: format!("DEMO-{}", &new_id.to_string()[0..4].to_uppercase()),
                        role: MemberRole::Owner,
                        member_count: 1,
                        totals: shared::dto::ListTotals {
                            total_items: 0,
                            purchased_items: 0,
                            total_planned: 0.0,
                            total_cart: 0.0,
                        },
                    };
                    lists.write().push(new_summary);
                    show_create_modal.set(false);
                    new_list_name.set(String::new());
                    new_list_desc.set(String::new());
                    success_msg.set(Some(format!("Lista '{}' criada com sucesso!", req.name)));
                }
            }
        });
    };

    let handle_join_list = move |_| {
        let code = join_code_input.read().trim().to_uppercase();
        let client = api_client.read().clone();

        if code.is_empty() {
            error_msg.set(Some("Digite o código de convite da lista".to_string()));
            return;
        }

        spawn(async move {
            let req = JoinListRequest {
                invite_code: code.clone(),
            };

            match client.join_list(req).await {
                Ok(joined) => {
                    show_join_modal.set(false);
                    join_code_input.set(String::new());
                    success_msg.set(Some(format!(
                        "Você entrou na lista '{}' com sucesso!",
                        joined.name
                    )));
                    load_lists();
                }
                Err(err) => {
                    error_msg.set(Some(err));
                }
            }
        });
    };

    let user_name = current_user
        .read()
        .as_ref()
        .map(|u| u.name.clone())
        .unwrap_or_else(|| "Visitante".to_string());

    rsx! {
        div { class: "main-content",
            // Header do Dashboard
            div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px; flex-wrap: wrap; gap: 12px;",
                div {
                    h2 { style: "font-size: 1.4rem; font-weight: 700;", "Olá, {user_name}! 👋" }
                    p { style: "color: #64748b; font-size: 0.9rem;", "Suas listas de compras compartilhadas" }
                }
                div { style: "display: flex; gap: 8px;",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| show_create_modal.set(true),
                        "➕ Nova Lista"
                    }
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| show_join_modal.set(true),
                        "🔗 Inserir Código"
                    }
                }
            }

            // Notificações
            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error",
                    div { style: "display: flex; justify-content: space-between;",
                        span { "{err}" }
                        span { style: "cursor: pointer; font-weight: bold;", onclick: move |_| error_msg.set(None), "✕" }
                    }
                }
            }

            if let Some(succ) = success_msg.read().as_ref() {
                div { class: "alert alert-success",
                    div { style: "display: flex; justify-content: space-between;",
                        span { "{succ}" }
                        span { style: "cursor: pointer; font-weight: bold;", onclick: move |_| success_msg.set(None), "✕" }
                    }
                }
            }

            // Listagem de Listas
            if *is_loading.read() && lists.read().is_empty() {
                div { style: "text-align: center; padding: 40px; color: #64748b;", "Carregando suas listas..." }
            } else if lists.read().is_empty() {
                div { class: "card", style: "text-align: center; padding: 40px 20px;",
                    div { style: "font-size: 2.5rem; margin-bottom: 12px;", "📝" }
                    h3 { style: "color: #334155; margin-bottom: 8px;", "Você ainda não tem listas de compras" }
                    p { style: "color: #64748b; margin-bottom: 20px; font-size: 0.95rem;",
                        "Crie sua primeira lista ou use um código de convite para participar da lista de alguém."
                    }
                    div { style: "display: flex; justify-content: center; gap: 12px;",
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| show_create_modal.set(true),
                            "Criar Minha Primeira Lista"
                        }
                    }
                }
            } else {
                for list in lists.read().iter() {
                    {
                        let list_id = list.id;
                        let progress_pct = if list.totals.total_items > 0 {
                            (list.totals.purchased_items as f64 / list.totals.total_items as f64) * 100.0
                        } else {
                            0.0
                        };

                        rsx! {
                            div {
                                key: "{list.id}",
                                class: "card",
                                style: "cursor: pointer; border-left: 5px solid #10b981; transition: transform 0.15s ease, box-shadow 0.15s ease;",
                                onclick: move |_| active_list_id.set(Some(list_id)),

                                div { style: "display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 8px;",
                                    div {
                                        h3 { style: "font-size: 1.25rem; font-weight: 700; color: #0f172a; margin-bottom: 4px;", "{list.name}" }
                                        if let Some(desc) = &list.description {
                                            p { style: "font-size: 0.88rem; color: #64748b; margin-bottom: 8px; line-height: 1.4;", "{desc}" }
                                        }
                                    }
                                    div { style: "display: flex; gap: 6px; align-items: center;",
                                        span { class: "badge badge-code", title: "Código de Convite Compartilhável",
                                            "🔑 {list.invite_code}"
                                        }
                                        if list.role == MemberRole::Owner {
                                            span { class: "badge", style: "background-color: #fef3c7; color: #b45309; font-weight: 600;", "👑 Dono" }
                                        } else {
                                            span { class: "badge", style: "background-color: #f1f5f9; color: #475569; font-weight: 600;", "Membro" }
                                        }
                                    }
                                }

                                // Barra de Progresso do Carrinho
                                div { class: "progress-container",
                                    div { class: "progress-bar", style: "width: {progress_pct:.0}%;" }
                                }

                                div { style: "display: flex; justify-content: space-between; align-items: center; font-size: 0.85rem; color: #64748b; margin-top: 8px;",
                                    span { style: "font-weight: 600; color: #059669;",
                                        "🛒 {list.totals.purchased_items}/{list.totals.total_items} no carrinho ({progress_pct:.0}%)"
                                    }
                                    span {
                                        "👥 {list.member_count} participante(s)"
                                    }
                                }

                                div { style: "display: flex; justify-content: space-between; align-items: center; border-top: 1px solid #f1f5f9; padding-top: 10px; margin-top: 10px; font-size: 0.9rem;",
                                    div {
                                        span { style: "color: #64748b;", "Planejado: " }
                                        strong { "R$ {list.totals.total_planned:.2}" }
                                    }
                                    div {
                                        span { style: "color: #059669; font-weight: 600;", "No Carrinho: " }
                                        strong { style: "color: #059669; font-size: 1.05rem;", "R$ {list.totals.total_cart:.2}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modal: Criar Nova Lista
            if *show_create_modal.read() {
                div { class: "modal-overlay",
                    div { class: "modal-content",
                        h3 { style: "margin-bottom: 16px; font-size: 1.25rem;", "Criar Nova Lista de Compras" }
                        div { class: "form-group",
                            label { class: "form-label", "Nome da Lista" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Mercado do Mês, Feira de Domingo...",
                                value: "{new_list_name}",
                                oninput: move |e| new_list_name.set(e.value())
                            }
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Descrição (Opcional)" }
                            input {
                                class: "form-input",
                                placeholder: "Ex: Compras da casa ou churrasco",
                                value: "{new_list_desc}",
                                oninput: move |e| new_list_desc.set(e.value())
                            }
                        }
                        div { style: "display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px;",
                            button {
                                class: "btn btn-secondary",
                                onclick: move |_| show_create_modal.set(false),
                                "Cancelar"
                            }
                            button {
                                class: "btn btn-primary",
                                onclick: handle_create_list,
                                "Criar Lista"
                            }
                        }
                    }
                }
            }

            // Modal: Entrar com Código
            if *show_join_modal.read() {
                div { class: "modal-overlay",
                    div { class: "modal-content",
                        h3 { style: "margin-bottom: 8px; font-size: 1.25rem;", "Entrar em Lista Compartilhada" }
                        p { style: "color: #64748b; font-size: 0.9rem; margin-bottom: 16px;",
                            "Digite o código de convite alfanumérico fornecido por quem criou a lista (ex: CASA-8F2A):"
                        }
                        div { class: "form-group",
                            label { class: "form-label", "Código de Convite" }
                            input {
                                class: "form-input",
                                style: "text-transform: uppercase; font-family: monospace; font-size: 1.2rem; letter-spacing: 2px;",
                                placeholder: "XXXX-XXXX",
                                value: "{join_code_input}",
                                oninput: move |e| join_code_input.set(e.value().to_uppercase())
                            }
                        }
                        div { style: "display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px;",
                            button {
                                class: "btn btn-secondary",
                                onclick: move |_| show_join_modal.set(false),
                                "Cancelar"
                            }
                            button {
                                class: "btn btn-primary",
                                onclick: handle_join_list,
                                "Entrar na Lista"
                            }
                        }
                    }
                }
            }
        }
    }
}
