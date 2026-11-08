use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use dioxus::prelude::*;
use shared::categories::CategoryId;
use shared::dto::{
    CreateItemRequest, FrequentItemResponse, ListDetailResponse, ToggleItemPurchasedRequest, UserProfile,
};
use shared::models::{Item, MemberRole, QuantityUnit};
use shared::ws::WsServerMessage;
use uuid::Uuid;

use crate::api::ApiClient;
use crate::format::{brl, parse_decimal, quantity as fmt_qty};
use crate::icons::{Icon, IconKind};
use crate::session::{show_api_error, sleep_ms, use_back_layer, Layer, LiveStatus, SessionExpired};
use crate::ws_client::{run_ws_sync, WsEvent};

/// Tempo para desfazer a remoção de um item
const UNDO_MS: u64 = 5_000;
/// Tempo de exibição dos avisos de outras pessoas
const TOAST_MS: u64 = 3_500;

/// Recalcula os totais e mantém a mesma ordem do servidor
/// (pendentes primeiro, mais recentes no topo)
fn normalize_detail(d: &mut ListDetailResponse) {
    d.items
        .sort_by(|a, b| a.purchased.cmp(&b.purchased).then(b.created_at.cmp(&a.created_at)));

    let mut total_planned = 0.0;
    let mut total_cart = 0.0;
    let mut purchased_count = 0;

    for item in d.items.iter() {
        let price = item.actual_price.or(item.estimated_price).unwrap_or(0.0);
        total_planned += price;
        if item.purchased {
            purchased_count += 1;
            total_cart += price;
        }
    }

    d.totals.total_items = d.items.len();
    d.totals.purchased_items = purchased_count;
    d.totals.total_planned = total_planned;
    d.totals.total_cart = total_cart;
}

/// Insere ou substitui um item na lista local
fn upsert_item(d: &mut ListDetailResponse, item: Item) {
    match d.items.iter().position(|i| i.id == item.id) {
        Some(pos) => d.items[pos] = item,
        None => d.items.push(item),
    }
    normalize_detail(d);
}

/// Copia um texto para a área de transferência
fn copy_to_clipboard(text: &str) {
    if let Ok(js_text) = serde_json::to_string(text) {
        let _ = document::eval(&format!(
            r#"
            (function (t) {{
                if (navigator.clipboard && navigator.clipboard.writeText) {{
                    navigator.clipboard.writeText(t).catch(function () {{}});
                    return;
                }}
                var el = document.createElement("textarea");
                el.value = t;
                el.setAttribute("readonly", "");
                el.style.position = "fixed";
                el.style.opacity = "0";
                document.body.appendChild(el);
                el.select();
                try {{ document.execCommand("copy"); }} catch (e) {{}}
                document.body.removeChild(el);
            }})({js_text});
            "#
        ));
    }
}

#[component]
pub fn ListDetailView(
    api_client: Signal<ApiClient>,
    current_user: Signal<Option<UserProfile>>,
    list_id: Uuid,
    active_list_id: Signal<Option<Uuid>>,
    error_msg: Signal<Option<String>>,
    success_msg: Signal<Option<String>>,
) -> Element {
    let session = use_context::<SessionExpired>();
    let live = use_context::<LiveStatus>();

    let mut detail = use_signal(|| None::<ListDetailResponse>);
    let mut is_loading = use_signal(|| true);
    let mut load_failed = use_signal(|| false);

    // Gaveta de sugestões (itens mais comprados)
    let mut show_suggestions = use_signal(|| false);
    let mut suggestions = use_signal(Vec::<FrequentItemResponse>::new);
    let mut loading_suggestions = use_signal(|| false);

    // Aviso temporário (ações de outras pessoas, código copiado...)
    let mut toast = use_signal(|| None::<String>);
    let mut toast_seq = use_signal(|| 0u64);

    // Item removido que ainda pode ser restaurado
    let mut undo_delete = use_signal(|| None::<(Item, Arc<AtomicBool>)>);

    // Gaveta de adição rápida
    let mut show_add_drawer = use_signal(|| false);
    let mut is_saving = use_signal(|| false);

    // Confirmação de exclusão da lista
    let mut show_delete_modal = use_signal(|| false);

    // O botão/gesto "voltar" do Android fecha cada camada
    use_back_layer(show_add_drawer, Layer::AddDrawer);
    use_back_layer(show_suggestions, Layer::Suggestions);
    use_back_layer(show_delete_modal, Layer::DeleteList);

    // Formulário de adição rápida
    let mut new_item_name = use_signal(String::new);
    let mut new_item_qty = use_signal(|| "1".to_string());
    let mut new_item_unit = use_signal(|| QuantityUnit::Un);
    let mut new_item_price = use_signal(String::new);
    let mut add_error = use_signal(|| None::<String>);

    let my_id = current_user.peek().as_ref().map(|u| u.id);

    let mut show_toast = move |message: String| {
        toast.set(Some(message));
        let seq = {
            let mut s = toast_seq.write();
            *s += 1;
            *s
        };
        spawn(async move {
            sleep_ms(TOAST_MS).await;
            if *toast_seq.peek() == seq {
                toast.set(None);
            }
        });
    };

    // Sai da tela quando a lista não existe mais ou o usuário perdeu o acesso
    let mut leave_list = move |message: String| {
        error_msg.set(Some(message));
        active_list_id.set(None);
    };

    // Carrega (ou recarrega) a lista completa do servidor
    let mut load_detail = move || {
        let client = api_client.peek().clone();
        is_loading.set(true);
        spawn(async move {
            match client.get_list_detail(list_id).await {
                Ok(mut data) => {
                    normalize_detail(&mut data);
                    detail.set(Some(data));
                    load_failed.set(false);
                }
                Err(err) if err.is_forbidden_or_missing() => leave_list(err.message),
                Err(err) => {
                    load_failed.set(true);
                    show_api_error(err, error_msg, session);
                }
            }
            is_loading.set(false);
        });
    };

    // Ao sair da tela, o indicador "Ao vivo" é desligado
    use_drop(move || {
        let mut status = live.0;
        status.set(false);
    });

    // Carregamento inicial + sincronização em tempo real (roda uma vez por lista)
    use_hook(move || {
        load_detail();

        let client = api_client.peek().clone();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WsEvent>();

        spawn(async move {
            run_ws_sync(client.base_url, list_id, client.token, tx).await;
        });

        spawn(async move {
            let mut status = live.0;
            let mut connected_before = false;

            while let Some(event) = rx.recv().await {
                let message = match event {
                    WsEvent::Connected => {
                        status.set(true);
                        // Após uma reconexão, eventos podem ter sido perdidos: recarrega
                        if connected_before {
                            load_detail();
                        }
                        connected_before = true;
                        continue;
                    }
                    WsEvent::Disconnected => {
                        status.set(false);
                        continue;
                    }
                    WsEvent::Message(m) => m,
                };

                // As próprias ações já estão na tela: não viram aviso
                let from_other = |actor: Uuid| Some(actor) != my_id;

                match message {
                    WsServerMessage::ItemAdded { item, actor_id, actor_name, .. } => {
                        if from_other(actor_id) {
                            show_toast(format!("{} adicionou {}", actor_name, item.name));
                        }
                        if let Some(d) = detail.write().as_mut() {
                            upsert_item(d, item);
                        }
                    }
                    WsServerMessage::ItemPurchasedToggled {
                        item_id,
                        purchased,
                        actual_price,
                        actor_id,
                        actor_name,
                        ..
                    } => {
                        let mut item_name = String::new();
                        if let Some(d) = detail.write().as_mut() {
                            if let Some(target) = d.items.iter_mut().find(|i| i.id == item_id) {
                                target.purchased = purchased;
                                target.actual_price = actual_price;
                                target.purchased_by_user_name = if purchased { Some(actor_name.clone()) } else { None };
                                item_name = target.name.clone();
                            }
                            normalize_detail(d);
                        }
                        if from_other(actor_id) && !item_name.is_empty() {
                            let action = if purchased { "pegou" } else { "devolveu" };
                            show_toast(format!("{} {} {}", actor_name, action, item_name));
                        }
                    }
                    WsServerMessage::ItemUpdated { item, actor_id, actor_name, .. } => {
                        if from_other(actor_id) {
                            show_toast(format!("{} editou {}", actor_name, item.name));
                        }
                        if let Some(d) = detail.write().as_mut() {
                            upsert_item(d, item);
                        }
                    }
                    WsServerMessage::ItemDeleted { item_id, actor_id, actor_name, .. } => {
                        if from_other(actor_id) {
                            show_toast(format!("{} removeu um item", actor_name));
                        }
                        if let Some(d) = detail.write().as_mut() {
                            d.items.retain(|i| i.id != item_id);
                            normalize_detail(d);
                        }
                    }
                    WsServerMessage::MemberJoined { member, .. } => {
                        show_toast(format!("{} entrou na lista", member.name));
                        if let Some(d) = detail.write().as_mut() {
                            if !d.members.iter().any(|m| m.user_id == member.user_id) {
                                d.members.push(member);
                            }
                        }
                    }
                    WsServerMessage::ListUpdated { name, description, .. } => {
                        if let Some(d) = detail.write().as_mut() {
                            d.list.name = name;
                            d.list.description = description;
                        }
                    }
                    WsServerMessage::ListDeleted { .. } => {
                        leave_list("Esta lista foi excluída pelo dono.".to_string());
                        break;
                    }
                    WsServerMessage::ResyncRequired { .. } => load_detail(),
                    _ => {}
                }
            }
        });
    });

    // Detecta a categoria em tempo real durante a digitação
    let current_name_input = new_item_name.read().clone();
    let preview_category = CategoryId::detect_from_name(&current_name_input);

    // Apenas o dono pode excluir a lista
    let is_owner = {
        let my_id = current_user.read().as_ref().map(|u| u.id);
        detail
            .read()
            .as_ref()
            .zip(my_id)
            .map(|(d, uid)| {
                d.members
                    .iter()
                    .any(|m| m.user_id == uid && m.role == MemberRole::Owner)
                    || d.list.created_by == uid
            })
            .unwrap_or(false)
    };

    let mut open_add_drawer = move || {
        add_error.set(None);
        show_add_drawer.set(true);
    };

    let mut handle_add_item = move || {
        if *is_saving.peek() {
            return;
        }
        let name = new_item_name.read().trim().to_string();
        if name.is_empty() {
            add_error.set(Some("Informe o nome do produto".to_string()));
            return;
        }

        let qty_text = new_item_qty.read().clone();
        let qty = if qty_text.trim().is_empty() {
            1.0
        } else {
            match parse_decimal(&qty_text) {
                Some(q) if q > 0.0 => q,
                _ => {
                    add_error.set(Some("Quantidade inválida. Use um número, como 1 ou 0,5".to_string()));
                    return;
                }
            }
        };

        let price_text = new_item_price.read().clone();
        let price_opt = if price_text.trim().is_empty() {
            None
        } else {
            match parse_decimal(&price_text) {
                Some(p) if p >= 0.0 => Some(p),
                _ => {
                    add_error.set(Some("Preço inválido. Use um valor como 12,90".to_string()));
                    return;
                }
            }
        };

        let unit = *new_item_unit.read();
        let client = api_client.peek().clone();
        is_saving.set(true);
        add_error.set(None);

        spawn(async move {
            let req = CreateItemRequest {
                category_id: Some(CategoryId::detect_from_name(&name)),
                name,
                quantity: Some(qty),
                unit: Some(unit),
                estimated_price: price_opt,
                notes: None,
            };

            match client.add_item(list_id, req).await {
                Ok(new_item) => {
                    new_item_name.set(String::new());
                    new_item_price.set(String::new());
                    new_item_qty.set("1".to_string());
                    show_add_drawer.set(false);
                    if let Some(d) = detail.write().as_mut() {
                        upsert_item(d, new_item);
                    }
                }
                // Nada é adicionado localmente: o formulário continua preenchido para tentar de novo
                Err(err) if err.is_unauthorized() => show_api_error(err, error_msg, session),
                Err(err) => add_error.set(Some(err.message)),
            }
            is_saving.set(false);
        });
    };

    // Marca/desmarca com atualização imediata na tela, desfeita se o servidor recusar
    let mut handle_toggle_item = move |item_id: Uuid, current_purchased: bool| {
        let client = api_client.peek().clone();

        let previous = detail
            .peek()
            .as_ref()
            .and_then(|d| d.items.iter().find(|i| i.id == item_id).cloned());
        let Some(previous) = previous else { return };

        if let Some(d) = detail.write().as_mut() {
            if let Some(target) = d.items.iter_mut().find(|i| i.id == item_id) {
                target.purchased = !current_purchased;
            }
            normalize_detail(d);
        }

        spawn(async move {
            let req = ToggleItemPurchasedRequest {
                purchased: !current_purchased,
                actual_price: None,
            };

            match client.toggle_item_purchased(list_id, item_id, req).await {
                Ok(updated_item) => {
                    if let Some(d) = detail.write().as_mut() {
                        upsert_item(d, updated_item);
                    }
                }
                Err(err) => {
                    if let Some(d) = detail.write().as_mut() {
                        upsert_item(d, previous);
                    }
                    show_api_error(err, error_msg, session);
                }
            }
        });
    };

    // Remove da tela na hora; o servidor só é chamado depois do prazo para desfazer
    let mut handle_delete_item = move |item_id: Uuid| {
        let removed = detail
            .peek()
            .as_ref()
            .and_then(|d| d.items.iter().find(|i| i.id == item_id).cloned());
        let Some(removed) = removed else { return };

        if let Some(d) = detail.write().as_mut() {
            d.items.retain(|i| i.id != item_id);
            normalize_detail(d);
        }

        let cancelled = Arc::new(AtomicBool::new(false));
        undo_delete.set(Some((removed.clone(), cancelled.clone())));

        let client = api_client.peek().clone();
        // Continua mesmo se a pessoa sair da tela antes do prazo
        let _ = spawn_forever(async move {
            sleep_ms(UNDO_MS).await;
            if cancelled.load(Ordering::SeqCst) {
                return;
            }
            // A tela pode ter sido fechada: só mexe nos sinais se ainda existirem
            if let Ok(mut pending) = undo_delete.try_write() {
                if pending.as_ref().map(|(i, _)| i.id) == Some(item_id) {
                    *pending = None;
                }
            }
            if let Err(err) = client.delete_item(list_id, item_id).await {
                if err.status == Some(404) {
                    return; // já tinha sido removido por outra pessoa
                }
                if let Ok(mut d) = detail.try_write() {
                    if let Some(d) = d.as_mut() {
                        upsert_item(d, removed);
                    }
                }
                if let Ok(mut msg) = error_msg.try_write() {
                    *msg = Some(err.message);
                }
            }
        });
    };

    let mut handle_undo = move || {
        if let Some((item, cancelled)) = undo_delete.take() {
            cancelled.store(true, Ordering::SeqCst);
            if let Some(d) = detail.write().as_mut() {
                upsert_item(d, item);
            }
        }
    };

    let open_suggestions = move |_| {
        let client = api_client.peek().clone();
        suggestions.set(Vec::new());
        loading_suggestions.set(true);
        show_suggestions.set(true);

        spawn(async move {
            match client.get_suggestions(list_id).await {
                Ok(data) => suggestions.set(data),
                Err(err) => {
                    show_suggestions.set(false);
                    show_api_error(err, error_msg, session);
                }
            }
            loading_suggestions.set(false);
        });
    };

    let add_suggested_item = move |item_name: String, cat: CategoryId, unit: QuantityUnit, price: Option<f64>| {
        let client = api_client.peek().clone();
        spawn(async move {
            let req = CreateItemRequest {
                name: item_name.clone(),
                category_id: Some(cat),
                quantity: Some(1.0),
                unit: Some(unit),
                estimated_price: price,
                notes: None,
            };

            match client.add_item(list_id, req).await {
                Ok(new_item) => {
                    show_toast(format!("{} adicionado", item_name));
                    if let Some(d) = detail.write().as_mut() {
                        upsert_item(d, new_item);
                    }
                }
                Err(err) => show_api_error(err, error_msg, session),
            }
        });
    };

    let handle_delete_list = move |_| {
        let client = api_client.peek().clone();
        show_delete_modal.set(false);
        spawn(async move {
            match client.delete_list(list_id).await {
                Ok(_) => {
                    success_msg.set(Some("Lista excluída".to_string()));
                    active_list_id.set(None);
                }
                Err(err) => show_api_error(err, error_msg, session),
            }
        });
    };

    rsx! {
        div { class: "main-content has-pill",
            // Barra superior: voltar, sugestões e código de convite
            div { class: "list-toolbar",
                button {
                    class: "btn btn-secondary btn-sm",
                    onclick: move |_| active_list_id.set(None),
                    Icon { kind: IconKind::ChevronLeft, size: 18 }
                    "Listas"
                }

                div { class: "toolbar-actions",
                    button {
                        class: "btn btn-secondary btn-sm",
                        onclick: open_suggestions,
                        "Sugeridos"
                    }
                    if let Some(d) = detail.read().as_ref() {
                        {
                            let code = d.list.invite_code.clone();
                            rsx! {
                                button {
                                    class: "btn btn-secondary btn-sm btn-code",
                                    "aria-label": "Copiar código de convite {code}",
                                    onclick: move |_| {
                                        copy_to_clipboard(&code);
                                        show_toast(format!("Código {} copiado. Envie para quem vai comprar junto.", code));
                                    },
                                    Icon { kind: IconKind::Copy, size: 16 }
                                    "{d.list.invite_code}"
                                }
                            }
                        }
                    }
                }
            }

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

            if let Some(d) = detail.read().as_ref() {
                {
                    let pending_items: Vec<Item> = d.items.iter().filter(|i| !i.purchased).cloned().collect();
                    let purchased_items: Vec<Item> = d.items.iter().filter(|i| i.purchased).cloned().collect();
                    // Em lista de uma pessoa só, "por Fulano" é ruído
                    let shared_list = d.members.len() > 1;

                    let progress_pct = if d.totals.total_items > 0 {
                        (d.totals.purchased_items as f64 / d.totals.total_items as f64) * 100.0
                    } else {
                        0.0
                    };

                    rsx! {
                        div { class: "list-header",
                            h1 { class: "list-title", "{d.list.name}" }
                            if let Some(desc) = &d.list.description {
                                p { class: "list-desc", "{desc}" }
                            }
                            if d.totals.total_items > 0 {
                                div { class: "list-progress-label",
                                    span { "{d.totals.purchased_items} de {d.totals.total_items} no carrinho" }
                                }
                                div {
                                    class: "progress",
                                    role: "progressbar",
                                    "aria-valuemin": "0",
                                    "aria-valuemax": "{d.totals.total_items}",
                                    "aria-valuenow": "{d.totals.purchased_items}",
                                    "aria-label": "Itens no carrinho",
                                    div { class: "progress-fill", style: "width: {progress_pct:.0}%;" }
                                }
                            }
                        }

                        if pending_items.is_empty() && purchased_items.is_empty() {
                            div { class: "empty-state",
                                h2 { "Lista vazia" }
                                p { "Toque no + lá embaixo para adicionar o primeiro produto." }
                            }
                        } else {
                            // Pendentes, agrupados por corredor
                            for cat in CategoryId::all().iter().filter(|c| pending_items.iter().any(|i| i.category_id == **c)) {
                                section { key: "{cat.as_str()}", class: "inset-group", "aria-label": "{cat.display_name()}",
                                    div { class: "aisle-header",
                                        span { class: "aisle-dot", style: "background-color: {cat.color_hex()};" }
                                        span { "{cat.display_name()}" }
                                    }

                                    for item in pending_items.iter().filter(|i| i.category_id == *cat) {
                                        {
                                            let item_id = item.id;
                                            rsx! {
                                                div {
                                                    key: "{item.id}",
                                                    class: "minimal-item-row",
                                                    role: "checkbox",
                                                    "aria-checked": "false",
                                                    tabindex: "0",
                                                    onclick: move |_| handle_toggle_item(item_id, false),
                                                    onkeydown: move |e: KeyboardEvent| {
                                                        if e.key() == Key::Enter || e.key() == Key::Character(" ".into()) {
                                                            e.prevent_default();
                                                            handle_toggle_item(item_id, false);
                                                        }
                                                    },

                                                    span { class: "check-circle", "aria-hidden": "true" }

                                                    div { class: "item-info",
                                                        div { class: "item-name", "{item.name}" }
                                                        div { class: "item-meta",
                                                            span { class: "item-qty-tag", "{fmt_qty(item.quantity)} {item.unit.as_str()}" }
                                                            if shared_list {
                                                                if let Some(adder) = &item.added_by_user_name {
                                                                    span { "por {adder}" }
                                                                }
                                                            }
                                                        }
                                                    }

                                                    if let Some(p) = item.actual_price.or(item.estimated_price) {
                                                        div { class: "item-price", "{brl(p)}" }
                                                    }

                                                    button {
                                                        class: "item-delete-btn",
                                                        "aria-label": "Remover {item.name}",
                                                        onclick: move |e: MouseEvent| {
                                                            e.stop_propagation();
                                                            handle_delete_item(item_id);
                                                        },
                                                        onkeydown: move |e: KeyboardEvent| e.stop_propagation(),
                                                        Icon { kind: IconKind::X, size: 18 }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // No carrinho
                            if !purchased_items.is_empty() {
                                h2 { class: "section-label", "No carrinho ({purchased_items.len()})" }

                                section { class: "inset-group", "aria-label": "No carrinho",
                                    for item in purchased_items.iter() {
                                        {
                                            let item_id = item.id;
                                            rsx! {
                                                div {
                                                    key: "{item.id}",
                                                    class: "minimal-item-row is-purchased",
                                                    role: "checkbox",
                                                    "aria-checked": "true",
                                                    tabindex: "0",
                                                    onclick: move |_| handle_toggle_item(item_id, true),
                                                    onkeydown: move |e: KeyboardEvent| {
                                                        if e.key() == Key::Enter || e.key() == Key::Character(" ".into()) {
                                                            e.prevent_default();
                                                            handle_toggle_item(item_id, true);
                                                        }
                                                    },

                                                    span { class: "check-circle checked", "aria-hidden": "true",
                                                        Icon { kind: IconKind::Check, size: 14, stroke: 3.0 }
                                                    }

                                                    div { class: "item-info",
                                                        div { class: "item-name", "{item.name}" }
                                                        div { class: "item-meta",
                                                            span { class: "item-qty-tag", "{fmt_qty(item.quantity)} {item.unit.as_str()}" }
                                                            if shared_list {
                                                                if let Some(buyer) = &item.purchased_by_user_name {
                                                                    span { "pego por {buyer}" }
                                                                }
                                                            }
                                                        }
                                                    }

                                                    if let Some(p) = item.actual_price.or(item.estimated_price) {
                                                        div { class: "item-price", "{brl(p)}" }
                                                    }

                                                    button {
                                                        class: "item-delete-btn",
                                                        "aria-label": "Remover {item.name}",
                                                        onclick: move |e: MouseEvent| {
                                                            e.stop_propagation();
                                                            handle_delete_item(item_id);
                                                        },
                                                        onkeydown: move |e: KeyboardEvent| e.stop_propagation(),
                                                        Icon { kind: IconKind::X, size: 18 }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Excluir a lista: um único lugar, discreto, no fim
                        if is_owner {
                            div { class: "danger-zone",
                                button {
                                    class: "btn btn-danger-text btn-sm",
                                    onclick: move |_| show_delete_modal.set(true),
                                    Icon { kind: IconKind::Trash, size: 16 }
                                    "Excluir lista"
                                }
                            }
                        }

                        // Pílula fixa: progresso, total no carrinho e adicionar
                        div { class: "floating-summary-pill",
                            div { class: "pill-summary", "aria-live": "polite",
                                span { class: "pill-count", "{d.totals.purchased_items}/{d.totals.total_items}" }
                                div { class: "pill-totals",
                                    strong { "{brl(d.totals.total_cart)}" }
                                    span { class: "pill-sub", "no carrinho · de {brl(d.totals.total_planned)}" }
                                }
                            }
                            button {
                                class: "floating-add-btn",
                                "aria-label": "Adicionar produto",
                                onclick: move |_| open_add_drawer(),
                                Icon { kind: IconKind::Plus, size: 24 }
                            }
                        }
                    }
                }
            } else if *is_loading.read() {
                div { class: "loading-state", "Carregando itens..." }
            } else if *load_failed.read() {
                div { class: "empty-state",
                    p { style: "margin-bottom: 16px;", "Não foi possível carregar a lista." }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| load_detail(),
                        "Tentar novamente"
                    }
                }
            }

            // Avisos temporários (não empurram a lista)
            div { class: "toast-stack", "aria-live": "polite",
                if let Some(msg) = toast.read().as_ref() {
                    div { class: "toast", role: "status",
                        span { class: "toast-text", "{msg}" }
                    }
                }
                if let Some((item, _)) = undo_delete.read().as_ref() {
                    div { class: "toast", role: "status",
                        span { class: "toast-text", "{item.name} removido" }
                        button {
                            class: "toast-action",
                            onclick: move |_| handle_undo(),
                            "Desfazer"
                        }
                    }
                }
            }

            // Gaveta: adicionar produto
            if *show_add_drawer.read() {
                div {
                    class: "drawer-overlay",
                    onclick: move |_| show_add_drawer.set(false),
                    div {
                        class: "drawer-content",
                        role: "dialog",
                        "aria-modal": "true",
                        "aria-labelledby": "add-title",
                        onclick: move |e| e.stop_propagation(),

                        div { class: "drawer-handle" }

                        div { class: "sheet-header",
                            h2 { id: "add-title", class: "sheet-title", "Adicionar produto" }
                            button {
                                class: "icon-btn",
                                "aria-label": "Fechar",
                                onclick: move |_| show_add_drawer.set(false),
                                Icon { kind: IconKind::X, size: 20 }
                            }
                        }

                        // Enter em qualquer campo adiciona
                        form {
                            onsubmit: move |e: FormEvent| {
                                e.prevent_default();
                                handle_add_item();
                            },

                            div { class: "form-group",
                                label { class: "visually-hidden", r#for: "new-item-name", "Nome do produto" }
                                input {
                                    id: "new-item-name",
                                    class: "form-input",
                                    autofocus: true,
                                    autocomplete: "off",
                                    enterkeyhint: "done",
                                    placeholder: "Nome do produto (ex: leite, maçã, picanha)",
                                    value: "{new_item_name}",
                                    oninput: move |e| new_item_name.set(e.value())
                                }
                            }

                            // Prévia da categoria (fundo claro da cor do corredor, texto escuro)
                            if !new_item_name.read().trim().is_empty() {
                                div { class: "category-preview",
                                    span { "Corredor:" }
                                    span {
                                        class: "category-chip",
                                        style: "background-color: {preview_category.color_hex()}26;",
                                        span { class: "aisle-dot", style: "background-color: {preview_category.color_hex()};" }
                                        "{preview_category.display_name()}"
                                    }
                                }
                            }

                            div { class: "form-row",
                                div { style: "flex: 1;",
                                    label { class: "form-label", r#for: "new-item-qty", "Qtd" }
                                    input {
                                        id: "new-item-qty",
                                        class: "form-input",
                                        r#type: "text",
                                        inputmode: "decimal",
                                        value: "{new_item_qty}",
                                        oninput: move |e| new_item_qty.set(e.value())
                                    }
                                }
                                div { style: "flex: 1.1;",
                                    label { class: "form-label", r#for: "new-item-unit", "Unidade" }
                                    select {
                                        id: "new-item-unit",
                                        class: "form-input",
                                        onchange: move |e| {
                                            new_item_unit.set(QuantityUnit::from_unit_str(&e.value()));
                                        },
                                        option { value: "un", "un" }
                                        option { value: "kg", "kg" }
                                        option { value: "g", "g" }
                                        option { value: "l", "l" }
                                        option { value: "ml", "ml" }
                                        option { value: "pct", "pct" }
                                        option { value: "cx", "cx" }
                                        option { value: "dz", "dz" }
                                    }
                                }
                                div { style: "flex: 1.5;",
                                    label { class: "form-label", r#for: "new-item-price", "Preço (R$)" }
                                    input {
                                        id: "new-item-price",
                                        class: "form-input",
                                        inputmode: "decimal",
                                        placeholder: "Opcional",
                                        value: "{new_item_price}",
                                        oninput: move |e| new_item_price.set(e.value())
                                    }
                                }
                            }

                            if let Some(err) = add_error.read().as_ref() {
                                p { class: "form-error", role: "alert", "{err}" }
                            }

                            button {
                                class: "btn btn-primary btn-block",
                                r#type: "submit",
                                disabled: *is_saving.read(),
                                if *is_saving.read() { "Adicionando..." } else { "Adicionar" }
                            }
                        }
                    }
                }
            }

            // Gaveta: sugestões (itens mais comprados nesta lista)
            if *show_suggestions.read() {
                div {
                    class: "drawer-overlay",
                    onclick: move |_| show_suggestions.set(false),
                    div {
                        class: "drawer-content",
                        role: "dialog",
                        "aria-modal": "true",
                        "aria-labelledby": "suggest-title",
                        onclick: move |e| e.stop_propagation(),

                        div { class: "drawer-handle" }

                        div { class: "sheet-header",
                            h2 { id: "suggest-title", class: "sheet-title", "Sugeridos" }
                            button {
                                class: "icon-btn",
                                "aria-label": "Fechar",
                                onclick: move |_| show_suggestions.set(false),
                                Icon { kind: IconKind::X, size: 20 }
                            }
                        }

                        {
                            // Não sugere o que já está pendente na lista
                            let pending_names: Vec<String> = detail
                                .read()
                                .as_ref()
                                .map(|d| {
                                    d.items
                                        .iter()
                                        .filter(|i| !i.purchased)
                                        .map(|i| i.name.trim().to_lowercase())
                                        .collect()
                                })
                                .unwrap_or_default();
                            let visible: Vec<FrequentItemResponse> = suggestions
                                .read()
                                .iter()
                                .filter(|s| !pending_names.contains(&s.name.trim().to_lowercase()))
                                .cloned()
                                .collect();

                            rsx! {
                                if *loading_suggestions.read() {
                                    p { class: "loading-state", "Carregando..." }
                                } else if visible.is_empty() {
                                    p { class: "sheet-hint",
                                        if suggestions.read().is_empty() {
                                            "Ainda não há histórico. Os produtos que vocês marcam no carrinho aparecem aqui nas próximas compras."
                                        } else {
                                            "Tudo o que vocês costumam comprar já está na lista."
                                        }
                                    }
                                } else {
                                    p { class: "sheet-hint", "O que vocês mais compram nesta lista:" }
                                    div { class: "suggestion-list",
                                        for item in visible.into_iter() {
                                            {
                                                let name = item.name.clone();
                                                let cat = item.category_id;
                                                let unit = item.last_unit;
                                                let price = item.last_price;
                                                let times = if item.purchase_count == 1 {
                                                    "1 vez".to_string()
                                                } else {
                                                    format!("{} vezes", item.purchase_count)
                                                };

                                                rsx! {
                                                    div { key: "{item.name}", class: "item-card",
                                                        div { style: "min-width: 0;",
                                                            div { class: "item-card-name", "{item.name}" }
                                                            div { class: "item-card-meta",
                                                                "Comprado {times}"
                                                                if let Some(p) = item.last_price {
                                                                    " · último {brl(p)}"
                                                                }
                                                            }
                                                        }
                                                        button {
                                                            class: "btn btn-secondary btn-sm",
                                                            "aria-label": "Adicionar {item.name}",
                                                            onclick: move |_| {
                                                                add_suggested_item(name.clone(), cat, unit, price);
                                                            },
                                                            Icon { kind: IconKind::Plus, size: 16 }
                                                            "Adicionar"
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Modal: confirmação de exclusão da lista
            if *show_delete_modal.read() {
                if let Some(d) = detail.read().as_ref() {
                    div { class: "modal-overlay",
                        div {
                            class: "modal-content",
                            role: "alertdialog",
                            "aria-modal": "true",
                            "aria-labelledby": "delete-title",
                            style: "max-width: 380px;",
                            h2 { id: "delete-title", class: "modal-title", "Excluir esta lista?" }
                            p { class: "modal-text",
                                strong { "{d.list.name}" }
                                " e todos os itens serão apagados para todos os participantes. Não dá para desfazer."
                            }
                            div { class: "modal-actions",
                                button {
                                    class: "btn btn-secondary",
                                    onclick: move |_| show_delete_modal.set(false),
                                    "Cancelar"
                                }
                                button {
                                    class: "btn btn-danger",
                                    onclick: handle_delete_list,
                                    "Excluir"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
