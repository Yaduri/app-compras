use dioxus::prelude::*;
use shared::dto::{CreateListRequest, JoinListRequest, ListSummaryResponse, UserProfile};
use uuid::Uuid;

use crate::api::{ApiClient, ApiError};
use crate::format::brl;
use crate::icons::{Icon, IconKind};
use crate::session::{show_api_error, use_back_layer, Layer, SessionExpired};

#[component]
pub fn DashboardView(
    api_client: Signal<ApiClient>,
    current_user: Signal<Option<UserProfile>>,
    active_list_id: Signal<Option<Uuid>>,
    error_msg: Signal<Option<String>>,
    success_msg: Signal<Option<String>>,
) -> Element {
    let _ = current_user;
    let mut lists = use_signal(Vec::<ListSummaryResponse>::new);
    let mut is_loading = use_signal(|| false);

    // Modais
    let mut show_create_modal = use_signal(|| false);
    let mut new_list_name = use_signal(String::new);
    let mut new_list_desc = use_signal(String::new);

    let mut show_join_modal = use_signal(|| false);
    let mut join_code_input = use_signal(String::new);

    // Erros de validação/servidor aparecem dentro do próprio modal
    let mut create_error = use_signal(|| None::<String>);
    let mut join_error = use_signal(|| None::<String>);

    let session = use_context::<SessionExpired>();
    let mut is_saving = use_signal(|| false);

    use_back_layer(show_create_modal, Layer::CreateList);
    use_back_layer(show_join_modal, Layer::JoinList);

    // Mostra o erro dentro do modal; sessão expirada continua levando ao login
    let modal_error = move |err: ApiError, mut target: Signal<Option<String>>| {
        if err.is_unauthorized() {
            show_api_error(err, error_msg, session);
        } else {
            target.set(Some(err.message));
        }
    };

    // Carrega listas do usuário (em caso de erro mantém o que já estava na tela e avisa)
    let mut load_lists = move || {
        let client = api_client.peek().clone();
        is_loading.set(true);
        spawn(async move {
            match client.list_user_lists().await {
                Ok(data) => lists.set(data),
                Err(err) => show_api_error(err, error_msg, session),
            }
            is_loading.set(false);
        });
    };

    use_effect(move || {
        load_lists();
    });

    let mut handle_create_list = move || {
        if *is_saving.peek() {
            return;
        }
        let name = new_list_name.read().trim().to_string();
        let desc = new_list_desc.read().trim().to_string();
        let client = api_client.peek().clone();

        if name.is_empty() {
            create_error.set(Some("Dê um nome para a lista".to_string()));
            return;
        }
        create_error.set(None);

        is_saving.set(true);
        spawn(async move {
            let req = CreateListRequest {
                name,
                description: if desc.is_empty() { None } else { Some(desc) },
            };

            match client.create_list(req).await {
                Ok(created) => {
                    show_create_modal.set(false);
                    new_list_name.set(String::new());
                    new_list_desc.set(String::new());
                    error_msg.set(None);
                    success_msg.set(None);
                    // Abre a lista recém-criada: o próximo passo é adicionar itens
                    active_list_id.set(Some(created.id));
                }
                // O modal continua aberto para o usuário tentar de novo
                Err(err) => modal_error(err, create_error),
            }
            is_saving.set(false);
        });
    };

    let mut handle_join_list = move || {
        if *is_saving.peek() {
            return;
        }
        let code = join_code_input.read().trim().to_uppercase();
        let client = api_client.peek().clone();

        if code.is_empty() {
            join_error.set(Some("Digite o código que você recebeu".to_string()));
            return;
        }
        join_error.set(None);

        is_saving.set(true);
        spawn(async move {
            let req = JoinListRequest { invite_code: code };

            match client.join_list(req).await {
                Ok(joined) => {
                    show_join_modal.set(false);
                    join_code_input.set(String::new());
                    success_msg.set(Some(format!("Você entrou na lista \"{}\"", joined.name)));
                    error_msg.set(None);
                    load_lists();
                }
                Err(err) => modal_error(err, join_error),
            }
            is_saving.set(false);
        });
    };

    let mut open_create = move || {
        create_error.set(None);
        show_create_modal.set(true);
    };

    rsx! {
        div { class: "main-content",
            div { class: "page-header",
                div {
                    h1 { class: "page-title", "Suas listas" }
                    p { class: "page-subtitle", "Compartilhe o código de uma lista para comprar junto" }
                }
                div { class: "page-actions",
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| open_create(),
                        Icon { kind: IconKind::Plus, size: 18 }
                        "Nova lista"
                    }
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            join_error.set(None);
                            show_join_modal.set(true);
                        },
                        "Entrar com código"
                    }
                }
            }

            // Notificações
            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error", role: "alert",
                    span { "{err}" }
                    button {
                        class: "icon-btn",
                        "aria-label": "Fechar aviso",
                        onclick: move |_| error_msg.set(None),
                        Icon { kind: IconKind::X, size: 18 }
                    }
                }
            }

            if let Some(succ) = success_msg.read().as_ref() {
                div { class: "alert alert-success", role: "status",
                    span { "{succ}" }
                    button {
                        class: "icon-btn",
                        "aria-label": "Fechar aviso",
                        onclick: move |_| success_msg.set(None),
                        Icon { kind: IconKind::X, size: 18 }
                    }
                }
            }

            // Listagem de listas
            if *is_loading.read() && lists.read().is_empty() {
                div { class: "loading-state", "Carregando suas listas..." }
            } else if lists.read().is_empty() {
                div { class: "card empty-state",
                    h2 { "Nenhuma lista ainda" }
                    p { style: "margin-bottom: 20px;",
                        "Crie sua primeira lista ou entre na lista de alguém com o código de convite."
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| open_create(),
                        Icon { kind: IconKind::Plus, size: 18 }
                        "Criar minha primeira lista"
                    }
                }
            } else {
                div { style: "display: flex; flex-direction: column; gap: 12px;",
                    for list in lists.read().iter() {
                        {
                            let list_id = list.id;
                            let progress_pct = if list.totals.total_items > 0 {
                                (list.totals.purchased_items as f64 / list.totals.total_items as f64) * 100.0
                            } else {
                                0.0
                            };
                            let others = list.member_count.saturating_sub(1);

                            rsx! {
                                button {
                                    key: "{list.id}",
                                    class: "card list-card",
                                    onclick: move |_| active_list_id.set(Some(list_id)),

                                    div { class: "list-card-head",
                                        div { style: "min-width: 0;",
                                            h2 { class: "list-card-title", "{list.name}" }
                                            if let Some(desc) = &list.description {
                                                p { class: "list-card-desc", "{desc}" }
                                            }
                                        }
                                        if others == 1 {
                                            span { class: "badge", "+1 pessoa" }
                                        } else if others > 1 {
                                            span { class: "badge", "+{others} pessoas" }
                                        }
                                    }

                                    div { class: "progress", "aria-hidden": "true",
                                        div { class: "progress-fill", style: "width: {progress_pct:.0}%;" }
                                    }

                                    div { class: "list-card-foot",
                                        span { "{list.totals.purchased_items} de {list.totals.total_items} no carrinho" }
                                        span { class: "list-card-total",
                                            strong { "{brl(list.totals.total_cart)}" }
                                            " de {brl(list.totals.total_planned)}"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modal: Criar nova lista
            if *show_create_modal.read() {
                div { class: "modal-overlay",
                    div {
                        class: "modal-content",
                        role: "dialog",
                        "aria-modal": "true",
                        "aria-labelledby": "create-title",
                        h2 { id: "create-title", class: "modal-title", "Nova lista" }
                        form {
                            onsubmit: move |e: FormEvent| {
                                e.prevent_default();
                                handle_create_list();
                            },
                            div { class: "form-group",
                                label { class: "form-label", r#for: "new-list-name", "Nome" }
                                input {
                                    id: "new-list-name",
                                    class: "form-input",
                                    autofocus: true,
                                    placeholder: "Ex: Mercado do mês, Feira de domingo",
                                    value: "{new_list_name}",
                                    oninput: move |e| new_list_name.set(e.value())
                                }
                            }
                            div { class: "form-group",
                                label { class: "form-label", r#for: "new-list-desc", "Descrição (opcional)" }
                                input {
                                    id: "new-list-desc",
                                    class: "form-input",
                                    placeholder: "Ex: Churrasco de sábado",
                                    value: "{new_list_desc}",
                                    oninput: move |e| new_list_desc.set(e.value())
                                }
                            }
                            if let Some(err) = create_error.read().as_ref() {
                                p { class: "form-error", role: "alert", "{err}" }
                            }
                            div { class: "modal-actions",
                                button {
                                    class: "btn btn-secondary",
                                    r#type: "button",
                                    onclick: move |_| show_create_modal.set(false),
                                    "Cancelar"
                                }
                                button {
                                    class: "btn btn-primary",
                                    r#type: "submit",
                                    disabled: *is_saving.read(),
                                    if *is_saving.read() { "Criando..." } else { "Criar lista" }
                                }
                            }
                        }
                    }
                }
            }

            // Modal: Entrar com código
            if *show_join_modal.read() {
                div { class: "modal-overlay",
                    div {
                        class: "modal-content",
                        role: "dialog",
                        "aria-modal": "true",
                        "aria-labelledby": "join-title",
                        h2 { id: "join-title", class: "modal-title", "Entrar em uma lista" }
                        p { class: "modal-text",
                            "Peça o código para quem criou a lista. Ele aparece no topo da lista, no formato CASA-8F2A."
                        }
                        form {
                            onsubmit: move |e: FormEvent| {
                                e.prevent_default();
                                handle_join_list();
                            },
                            div { class: "form-group",
                                label { class: "form-label", r#for: "join-code", "Código de convite" }
                                input {
                                    id: "join-code",
                                    class: "form-input code-input",
                                    autofocus: true,
                                    autocomplete: "off",
                                    autocapitalize: "characters",
                                    placeholder: "XXXX-XXXX",
                                    value: "{join_code_input}",
                                    oninput: move |e| join_code_input.set(e.value().to_uppercase())
                                }
                            }
                            if let Some(err) = join_error.read().as_ref() {
                                p { class: "form-error", role: "alert", "{err}" }
                            }
                            div { class: "modal-actions",
                                button {
                                    class: "btn btn-secondary",
                                    r#type: "button",
                                    onclick: move |_| show_join_modal.set(false),
                                    "Cancelar"
                                }
                                button {
                                    class: "btn btn-primary",
                                    r#type: "submit",
                                    disabled: *is_saving.read(),
                                    if *is_saving.read() { "Entrando..." } else { "Entrar na lista" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
