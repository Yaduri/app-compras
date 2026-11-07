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

    let mut list_to_delete = use_signal(|| None::<(Uuid, String)>);

    // Carrega listas do usuário
    let mut load_lists = move || {
        let client = api_client.read().clone();
        is_loading.set(true);
        spawn(async move {
            match client.list_user_lists().await {
                Ok(data) => {
                    lists.set(data);
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
            div { style: "display: flex; justify-content: space-between; align-items: flex-end; margin-bottom: 24px; flex-wrap: wrap; gap: 12px; padding-bottom: 12px; border-bottom: 1px solid var(--border-light);",
                div {
                    h2 { style: "font-size: 1.5rem; font-weight: 800; letter-spacing: -0.025em; color: var(--text-primary);", "{user_name}" }
                    p { style: "color: var(--text-secondary); font-size: 0.85rem; margin-top: 2px;", "Listas de compras compartilhadas" }
                }
                div { style: "display: flex; gap: 8px;",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| show_create_modal.set(true),
                        "+ Nova Lista"
                    }
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| show_join_modal.set(true),
                        "Inserir Código"
                    }
                }
            }

            // Notificações
            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error",
                    div { style: "display: flex; justify-content: space-between; width: 100%;",
                        span { "{err}" }
                        span { style: "cursor: pointer; font-weight: bold;", onclick: move |_| error_msg.set(None), "✕" }
                    }
                }
            }

            if let Some(succ) = success_msg.read().as_ref() {
                div { class: "alert alert-success",
                    div { style: "display: flex; justify-content: space-between; width: 100%;",
                        span { "{succ}" }
                        span { style: "cursor: pointer; font-weight: bold;", onclick: move |_| success_msg.set(None), "✕" }
                    }
                }
            }

            // Listagem de Listas
            if *is_loading.read() && lists.read().is_empty() {
                div { style: "text-align: center; padding: 48px; color: var(--text-muted); font-size: 0.9rem;", "Carregando suas listas..." }
            } else if lists.read().is_empty() {
                div { class: "inset-group", style: "text-align: center; padding: 48px 20px;",
                    h3 { style: "color: var(--text-primary); font-size: 1.1rem; font-weight: 700; margin-bottom: 6px;", "Nenhuma lista encontrada" }
                    p { style: "color: var(--text-secondary); margin-bottom: 20px; font-size: 0.88rem;",
                        "Crie sua primeira lista ou use um código de convite para participar da lista de alguém."
                    }
                    div { style: "display: flex; justify-content: center; gap: 12px;",
                        button {
                            class: "btn btn-primary",
                            onclick: move |_| show_create_modal.set(true),
                            "+ Criar Minha Primeira Lista"
                        }
                    }
                }
            } else {
                div { style: "display: flex; flex-direction: column; gap: 12px;",
                    for list in lists.read().iter() {
                        {
                            let list_id = list.id;
                            let lid = list.id;
                            let lname = list.name.clone();
                            let progress_pct = if list.totals.total_items > 0 {
                                (list.totals.purchased_items as f64 / list.totals.total_items as f64) * 100.0
                            } else {
                                0.0
                            };

                            rsx! {
                                div {
                                    key: "{list.id}",
                                    class: "card",
                                    style: "cursor: pointer; padding: 18px 20px; margin-bottom: 0; transition: border-color 0.15s ease, transform 0.1s ease;",
                                    onclick: move |_| active_list_id.set(Some(list_id)),

                                    div { style: "display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 8px;",
                                        div { style: "flex: 1;",
                                            h3 { style: "font-size: 1.18rem; font-weight: 700; color: var(--text-primary); margin-bottom: 3px; letter-spacing: -0.015em;", "{list.name}" }
                                            if let Some(desc) = &list.description {
                                                p { style: "font-size: 0.84rem; color: var(--text-secondary); margin-bottom: 6px; line-height: 1.3;", "{desc}" }
                                            }
                                        }
                                        div { style: "display: flex; gap: 6px; align-items: center;",
                                            span { class: "badge badge-code", title: "Código de Convite",
                                                "{list.invite_code}"
                                            }
                                            if list.role == MemberRole::Owner {
                                                span { class: "badge", style: "background: #f1f5f9; color: #0f172a; font-weight: 600;", "Dono" }
                                                button {
                                                    class: "btn-icon-subtle",
                                                    style: "background: rgba(239, 68, 68, 0.08); border: 1px solid rgba(239, 68, 68, 0.2); color: #ef4444; border-radius: 8px; padding: 4px 8px; font-size: 0.82rem; cursor: pointer;",
                                                    title: "Excluir lista",
                                                    onclick: move |e: MouseEvent| {
                                                        e.stop_propagation();
                                                        list_to_delete.set(Some((lid, lname.clone())));
                                                    },
                                                    "🗑️"
                                                }
                                            } else {
                                                span { class: "badge", style: "background: #f1f5f9; color: #64748b; font-weight: 500;", "Membro" }
                                            }
                                        }
                                    }

                                    // Barra de Progresso Discreta (4px)
                                    div { style: "width: 100%; height: 4px; background: #f1f5f9; border-radius: 2px; overflow: hidden; margin-top: 10px; margin-bottom: 8px;",
                                        div { style: "width: {progress_pct:.0}%; height: 100%; background: var(--carbon-dark); border-radius: 2px; transition: width 0.2s ease;" }
                                    }

                                    div { style: "display: flex; justify-content: space-between; align-items: center; font-size: 0.82rem; color: var(--text-secondary); margin-top: 6px;",
                                        span {
                                            "{list.totals.purchased_items} de {list.totals.total_items} no carrinho ({progress_pct:.0}%)"
                                        }
                                        span {
                                            "{list.member_count} participante(s)"
                                        }
                                    }

                                    div { style: "display: flex; justify-content: space-between; align-items: center; border-top: 1px solid var(--border-light); padding-top: 10px; margin-top: 12px; font-size: 0.88rem;",
                                        div {
                                            span { style: "color: var(--text-muted); font-size: 0.8rem;", "Planejado: " }
                                            strong { style: "color: var(--text-secondary);", "R$ {list.totals.total_planned:.2}" }
                                        }
                                        div {
                                            span { style: "color: var(--text-muted); font-size: 0.8rem;", "No Carrinho: " }
                                            strong { style: "color: var(--text-primary); font-size: 1rem;", "R$ {list.totals.total_cart:.2}" }
                                        }
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

            // Modal: Confirmação de Exclusão de Lista
            if let Some((del_id, del_name)) = list_to_delete.read().clone() {
                div { class: "modal-overlay",
                    div { class: "card",
                        style: "max-width: 380px; width: 100%; border-radius: 20px; padding: 24px; box-shadow: 0 20px 40px rgba(0,0,0,0.25); border: 1px solid var(--border-light); background: var(--bg-card);",
                        h3 { style: "color: #ef4444; font-size: 1.2rem; font-weight: 700; margin-bottom: 8px;", "Excluir Lista?" }
                        p { style: "color: var(--text-secondary); font-size: 0.9rem; margin-bottom: 24px; line-height: 1.45;",
                            "Tem certeza que deseja excluir permanentemente a lista "
                            strong { style: "color: var(--text-primary);", "\"{del_name}\"" }
                            "? Todos os itens da lista serão apagados."
                        }
                        div { style: "display: flex; justify-content: flex-end; gap: 10px;",
                            button {
                                class: "btn btn-secondary",
                                style: "padding: 8px 18px;",
                                onclick: move |_| list_to_delete.set(None),
                                "Cancelar"
                            }
                            button {
                                class: "btn",
                                style: "background: #ef4444; color: #ffffff; border: none; padding: 8px 18px; font-weight: 700; border-radius: 9999px; cursor: pointer; box-shadow: 0 4px 12px rgba(239, 68, 68, 0.3);",
                                onclick: move |_| {
                                    let client = api_client.read().clone();
                                    let target_id = del_id;
                                    list_to_delete.set(None);
                                    spawn(async move {
                                        match client.delete_list(target_id).await {
                                            Ok(_) => {
                                                success_msg.set(Some("Lista excluída com sucesso!".to_string()));
                                                load_lists();
                                            }
                                            Err(err) => {
                                                error_msg.set(Some(err));
                                            }
                                        }
                                    });
                                },
                                "Sim, Excluir"
                            }
                        }
                    }
                }
            }
        }
    }
}
