use dioxus::prelude::*;
use shared::categories::CategoryId;
use shared::dto::{
    CreateItemRequest, FrequentItemResponse, ListDetailResponse, ToggleItemPurchasedRequest,
};
use shared::models::{Item, QuantityUnit};
use shared::ws::WsServerMessage;
use uuid::Uuid;

use crate::api::ApiClient;
use crate::ws_client::start_ws_sync;

// Função pura de recálculo dos totais da lista
fn recalculate_totals(d: &mut ListDetailResponse) {
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

#[component]
pub fn ListDetailView(
    api_client: Signal<ApiClient>,
    list_id: Uuid,
    active_list_id: Signal<Option<Uuid>>,
    error_msg: Signal<Option<String>>,
    success_msg: Signal<Option<String>>,
) -> Element {
    let mut detail = use_signal(|| None::<ListDetailResponse>);
    let mut is_loading = use_signal(|| true);

    // Modal de Sugestões / Mais Comprados
    let mut show_suggestions_modal = use_signal(|| false);
    let mut suggestions = use_signal(Vec::<FrequentItemResponse>::new);
    let mut loading_suggestions = use_signal(|| false);

    // Notificação em tempo real (ex: "Maria marcou Leite no carrinho")
    let mut real_time_alert = use_signal(|| None::<String>);

    // Gaveta / Bottom Sheet de Adição Rápida
    let mut show_add_drawer = use_signal(|| false);

    // Formulário de adição rápida
    let mut new_item_name = use_signal(String::new);
    let mut new_item_qty = use_signal(|| "1".to_string());
    let mut new_item_unit = use_signal(|| QuantityUnit::Un);
    let mut new_item_price = use_signal(String::new);

    // Carregamento inicial da lista
    let mut load_detail = move || {
        let client = api_client.read().clone();
        is_loading.set(true);
        spawn(async move {
            match client.get_list_detail(list_id).await {
                Ok(data) => {
                    detail.set(Some(data));
                }
                Err(_) => {
                    // Fallback para lista demo se backend indisponível
                    let demo_id = Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap();
                    let target_id = if list_id == demo_id { demo_id } else { list_id };
                    let sample_items = vec![
                        Item {
                            id: Uuid::new_v4(),
                            list_id: target_id,
                            category_id: CategoryId::Butcher,
                            name: "Picanha Bovina 1.5kg".to_string(),
                            quantity: 1.5,
                            unit: QuantityUnit::Kg,
                            estimated_price: Some(115.00),
                            actual_price: None,
                            purchased: false,
                            purchased_at: None,
                            purchased_by_user_id: None,
                            purchased_by_user_name: None,
                            added_by_user_id: Uuid::nil(),
                            added_by_user_name: Some("Usuário Demo".to_string()),
                            notes: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        },
                        Item {
                            id: Uuid::new_v4(),
                            list_id: target_id,
                            category_id: CategoryId::Butcher,
                            name: "Linguiça Toscana".to_string(),
                            quantity: 1.0,
                            unit: QuantityUnit::Kg,
                            estimated_price: Some(24.90),
                            actual_price: Some(23.50),
                            purchased: true,
                            purchased_at: Some(chrono::Utc::now()),
                            purchased_by_user_id: None,
                            purchased_by_user_name: Some("Usuário Demo".to_string()),
                            added_by_user_id: Uuid::nil(),
                            added_by_user_name: Some("Usuário Demo".to_string()),
                            notes: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        },
                        Item {
                            id: Uuid::new_v4(),
                            list_id: target_id,
                            category_id: CategoryId::Produce,
                            name: "Carvão Vegetal 5kg".to_string(),
                            quantity: 1.0,
                            unit: QuantityUnit::Pct,
                            estimated_price: Some(26.00),
                            actual_price: Some(24.80),
                            purchased: true,
                            purchased_at: Some(chrono::Utc::now()),
                            purchased_by_user_id: None,
                            purchased_by_user_name: Some("Usuário Demo".to_string()),
                            added_by_user_id: Uuid::nil(),
                            added_by_user_name: Some("Usuário Demo".to_string()),
                            notes: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        },
                        Item {
                            id: Uuid::new_v4(),
                            list_id: target_id,
                            category_id: CategoryId::Produce,
                            name: "Pão de Alho Tradicional".to_string(),
                            quantity: 2.0,
                            unit: QuantityUnit::Pct,
                            estimated_price: Some(18.00),
                            actual_price: None,
                            purchased: false,
                            purchased_at: None,
                            purchased_by_user_id: None,
                            purchased_by_user_name: None,
                            added_by_user_id: Uuid::nil(),
                            added_by_user_name: Some("Usuário Demo".to_string()),
                            notes: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        },
                        Item {
                            id: Uuid::new_v4(),
                            list_id: target_id,
                            category_id: CategoryId::Produce,
                            name: "Cerveja Lata 350ml (Pack com 12)".to_string(),
                            quantity: 1.0,
                            unit: QuantityUnit::Cx,
                            estimated_price: Some(42.50),
                            actual_price: None,
                            purchased: false,
                            purchased_at: None,
                            purchased_by_user_id: None,
                            purchased_by_user_name: None,
                            added_by_user_id: Uuid::nil(),
                            added_by_user_name: Some("Usuário Demo".to_string()),
                            notes: None,
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        },
                    ];
                    let mut resp = ListDetailResponse {
                        list: shared::models::List {
                            id: target_id,
                            name: "Churrasco do Fim de Semana".to_string(),
                            description: Some("Lista colaborativa para o churrasco com a família e amigos".to_string()),
                            invite_code: "CHURRAS".to_string(),
                            created_by: Uuid::nil(),
                            created_at: chrono::Utc::now(),
                            updated_at: chrono::Utc::now(),
                        },
                        members: vec![],
                        items: sample_items,
                        totals: shared::dto::ListTotals {
                            total_items: 5,
                            purchased_items: 2,
                            total_planned: 226.40,
                            total_cart: 48.30,
                        },
                    };
                    recalculate_totals(&mut resp);
                    detail.set(Some(resp));
                }
            }
            is_loading.set(false);
        });
    };

    // Inicialização e WebSocket Sync
    use_effect(move || {
        load_detail();

        // Inicia sincronização em tempo real via WebSocket usando canal seguro MPSC
        let client = api_client.read().clone();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<WsServerMessage>();

        spawn(async move {
            start_ws_sync(client.base_url, list_id, client.token, tx).await;
        });

        spawn(async move {
            while let Some(event) = rx.recv().await {
                match event {
                    WsServerMessage::ItemAdded { item, actor_name, .. } => {
                        real_time_alert.set(Some(format!("✨ {} adicionou '{}'", actor_name, item.name)));
                        if let Some(d) = detail.write().as_mut() {
                            if !d.items.iter().any(|i| i.id == item.id) {
                                d.items.push(item);
                            }
                            recalculate_totals(d);
                        }
                    }
                    WsServerMessage::ItemPurchasedToggled {
                        item_id,
                        purchased,
                        actual_price,
                        actor_name,
                        ..
                    } => {
                        let mut item_name_affected = String::new();
                        if let Some(d) = detail.write().as_mut() {
                            if let Some(target) = d.items.iter_mut().find(|i| i.id == item_id) {
                                target.purchased = purchased;
                                target.actual_price = actual_price;
                                target.purchased_by_user_name = if purchased {
                                    Some(actor_name.clone())
                                } else {
                                    None
                                };
                                item_name_affected = target.name.clone();
                            }
                            recalculate_totals(d);
                        }

                        let action_text = if purchased { "marcou no carrinho" } else { "desmarcou" };
                        real_time_alert.set(Some(format!("🛒 {} {} '{}'", actor_name, action_text, item_name_affected)));
                    }
                    WsServerMessage::ItemUpdated { item, actor_name, .. } => {
                        real_time_alert.set(Some(format!("✏️ {} editou '{}'", actor_name, item.name)));
                        if let Some(d) = detail.write().as_mut() {
                            if let Some(pos) = d.items.iter().position(|i| i.id == item.id) {
                                d.items[pos] = item;
                            }
                            recalculate_totals(d);
                        }
                    }
                    WsServerMessage::ItemDeleted { item_id, actor_name, .. } => {
                        real_time_alert.set(Some(format!("🗑️ {} removeu um item", actor_name)));
                        if let Some(d) = detail.write().as_mut() {
                            d.items.retain(|i| i.id != item_id);
                            recalculate_totals(d);
                        }
                    }
                    WsServerMessage::MemberJoined { member, .. } => {
                        real_time_alert.set(Some(format!("👋 {} entrou na lista!", member.name)));
                        if let Some(d) = detail.write().as_mut() {
                            if !d.members.iter().any(|m| m.user_id == member.user_id) {
                                d.members.push(member);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
    });

    // Detecta a categoria em tempo real durante a digitação
    let current_name_input = new_item_name.read().clone();
    let preview_category = CategoryId::detect_from_name(&current_name_input);

    let handle_add_item = move |_| {
        let name = new_item_name.read().trim().to_string();
        let qty: f64 = new_item_qty.read().trim().replace(',', ".").parse().unwrap_or(1.0);
        let unit = *new_item_unit.read();
        let price_opt: Option<f64> = new_item_price.read().trim().replace(',', ".").parse().ok();
        let client = api_client.read().clone();

        if name.is_empty() {
            error_msg.set(Some("Informe o nome do produto".to_string()));
            return;
        }

        spawn(async move {
            let req = CreateItemRequest {
                name: name.clone(),
                category_id: Some(CategoryId::detect_from_name(&name)),
                quantity: Some(qty),
                unit: Some(unit),
                estimated_price: price_opt,
                notes: None,
            };

            match client.add_item(list_id, req.clone()).await {
                Ok(new_item) => {
                    new_item_name.set(String::new());
                    new_item_price.set(String::new());
                    new_item_qty.set("1".to_string());
                    show_add_drawer.set(false);
                    if let Some(d) = detail.write().as_mut() {
                        if !d.items.iter().any(|i| i.id == new_item.id) {
                            d.items.push(new_item);
                        }
                        recalculate_totals(d);
                    }
                }
                Err(_) => {
                    // Fallback para modo demo / offline
                    let fallback_item = Item {
                        id: Uuid::new_v4(),
                        list_id,
                        category_id: req.category_id.unwrap_or(CategoryId::Grocery),
                        name: req.name,
                        quantity: req.quantity.unwrap_or(1.0),
                        unit: req.unit.unwrap_or(QuantityUnit::Un),
                        estimated_price: req.estimated_price,
                        actual_price: None,
                        purchased: false,
                        purchased_by_user_id: None,
                        purchased_by_user_name: None,
                        added_by_user_id: Uuid::nil(),
                        added_by_user_name: Some("Você (Demo)".to_string()),
                        notes: req.notes,
                        purchased_at: None,
                        created_at: chrono::Utc::now(),
                        updated_at: chrono::Utc::now(),
                    };
                    new_item_name.set(String::new());
                    new_item_price.set(String::new());
                    new_item_qty.set("1".to_string());
                    show_add_drawer.set(false);
                    if let Some(d) = detail.write().as_mut() {
                        d.items.push(fallback_item);
                        recalculate_totals(d);
                    }
                }
            }
        });
    };

    let handle_toggle_item = move |item_id: Uuid, current_purchased: bool, current_price: Option<f64>| {
        let client = api_client.read().clone();
        spawn(async move {
            let req = ToggleItemPurchasedRequest {
                purchased: !current_purchased,
                actual_price: current_price,
            };

            match client.toggle_item_purchased(list_id, item_id, req).await {
                Ok(updated_item) => {
                    if let Some(d) = detail.write().as_mut() {
                        if let Some(target) = d.items.iter_mut().find(|i| i.id == item_id) {
                            *target = updated_item;
                        }
                        recalculate_totals(d);
                    }
                }
                Err(_) => {
                    // Fallback local se backend offline
                    if let Some(d) = detail.write().as_mut() {
                        if let Some(target) = d.items.iter_mut().find(|i| i.id == item_id) {
                            target.purchased = !current_purchased;
                            target.actual_price = if target.purchased { current_price } else { None };
                        }
                        recalculate_totals(d);
                    }
                }
            }
        });
    };

    let handle_delete_item = move |item_id: Uuid| {
        let client = api_client.read().clone();
        spawn(async move {
            let _ = client.delete_item(list_id, item_id).await;
            if let Some(d) = detail.write().as_mut() {
                d.items.retain(|i| i.id != item_id);
                recalculate_totals(d);
            }
        });
    };

    let open_suggestions = move |_| {
        let client = api_client.read().clone();
        loading_suggestions.set(true);
        show_suggestions_modal.set(true);

        spawn(async move {
            match client.get_suggestions(list_id).await {
                Ok(data) => {
                    if data.is_empty() {
                        suggestions.set(vec![
                            FrequentItemResponse {
                                name: "Pão de Alho".to_string(),
                                category_id: CategoryId::Produce,
                                purchase_count: 5,
                                last_unit: QuantityUnit::Pct,
                                last_price: Some(18.00),
                            },
                            FrequentItemResponse {
                                name: "Carvão Vegetal".to_string(),
                                category_id: CategoryId::Produce,
                                purchase_count: 4,
                                last_unit: QuantityUnit::Pct,
                                last_price: Some(25.00),
                            },
                            FrequentItemResponse {
                                name: "Queijo Coalho".to_string(),
                                category_id: CategoryId::Dairy,
                                purchase_count: 3,
                                last_unit: QuantityUnit::Pct,
                                last_price: Some(19.90),
                            },
                            FrequentItemResponse {
                                name: "Sal Grosso".to_string(),
                                category_id: CategoryId::Grocery,
                                purchase_count: 3,
                                last_unit: QuantityUnit::Kg,
                                last_price: Some(6.50),
                            },
                        ]);
                    } else {
                        suggestions.set(data);
                    }
                }
                Err(_) => {
                    suggestions.set(vec![
                        FrequentItemResponse {
                            name: "Pão de Alho".to_string(),
                            category_id: CategoryId::Produce,
                            purchase_count: 5,
                            last_unit: QuantityUnit::Pct,
                            last_price: Some(18.00),
                        },
                        FrequentItemResponse {
                            name: "Carvão Vegetal".to_string(),
                            category_id: CategoryId::Produce,
                            purchase_count: 4,
                            last_unit: QuantityUnit::Pct,
                            last_price: Some(25.00),
                        },
                        FrequentItemResponse {
                            name: "Queijo Coalho".to_string(),
                            category_id: CategoryId::Dairy,
                            purchase_count: 3,
                            last_unit: QuantityUnit::Pct,
                            last_price: Some(19.90),
                        },
                        FrequentItemResponse {
                            name: "Sal Grosso".to_string(),
                            category_id: CategoryId::Grocery,
                            purchase_count: 3,
                            last_unit: QuantityUnit::Kg,
                            last_price: Some(6.50),
                        },
                    ]);
                }
            }
            loading_suggestions.set(false);
        });
    };

    let add_suggested_item = move |item_name: String, cat: CategoryId, unit: QuantityUnit, price: Option<f64>| {
        let client = api_client.read().clone();
        spawn(async move {
            let req = CreateItemRequest {
                name: item_name,
                category_id: Some(cat),
                quantity: Some(1.0),
                unit: Some(unit),
                estimated_price: price,
                notes: None,
            };

            if let Ok(new_item) = client.add_item(list_id, req).await {
                if let Some(d) = detail.write().as_mut() {
                    if !d.items.iter().any(|i| i.id == new_item.id) {
                        d.items.push(new_item);
                    }
                    recalculate_totals(d);
                }
            }
        });
    };

    rsx! {
        div { class: "main-content",
            // Barra Superior de Navegação
            div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; gap: 8px;",
                button {
                    class: "btn btn-secondary",
                    style: "padding: 6px 14px; font-size: 0.85rem;",
                    onclick: move |_| active_list_id.set(None),
                    "← Listas"
                }

                div { style: "display: flex; align-items: center; gap: 8px;",
                    button {
                        class: "btn btn-secondary",
                        style: "padding: 6px 12px; font-size: 0.82rem;",
                        onclick: open_suggestions,
                        "Sugeridos"
                    }
                    if let Some(d) = detail.read().as_ref() {
                        span {
                            class: "badge badge-code",
                            title: "Código da Lista",
                            "{d.list.invite_code}"
                        }
                    }
                }
            }

            // Notificações em Tempo Real (WebSocket)
            if let Some(msg) = real_time_alert.read().as_ref() {
                div { class: "alert alert-success",
                    span { "{msg}" }
                    span {
                        style: "cursor: pointer; font-weight: bold; margin-left: 12px;",
                        onclick: move |_| real_time_alert.set(None),
                        "✕"
                    }
                }
            }

            if let Some(err) = error_msg.read().as_ref() {
                div { class: "alert alert-error", "{err}" }
            }

            if let Some(d) = detail.read().as_ref() {
                {
                    let pending_items: Vec<Item> = d.items.iter().filter(|i| !i.purchased).cloned().collect();
                    let purchased_items: Vec<Item> = d.items.iter().filter(|i| i.purchased).cloned().collect();

                    let progress_pct = if d.totals.total_items > 0 {
                        (d.totals.purchased_items as f64 / d.totals.total_items as f64) * 100.0
                    } else {
                        0.0
                    };

                    rsx! {
                        // Cabeçalho Editorial Minimalista (Things 3)
                        div { style: "margin-top: 8px; margin-bottom: 20px;",
                            h1 { style: "font-size: 1.7rem; font-weight: 800; letter-spacing: -0.025em; color: var(--text-primary); margin-bottom: 4px;", "{d.list.name}" }
                            if let Some(desc) = &d.list.description {
                                p { style: "color: var(--text-secondary); font-size: 0.88rem; margin-bottom: 12px;", "{desc}" }
                            }

                            // Barra de Progresso Discreta (4px)
                            div { style: "display: flex; justify-content: space-between; align-items: center; font-size: 0.8rem; color: var(--text-secondary); margin-bottom: 6px; margin-top: 10px;",
                                span { "{d.totals.purchased_items} de {d.totals.total_items} no carrinho" }
                                span { style: "font-weight: 600; color: var(--text-primary);", "{progress_pct:.0}%" }
                            }
                            div { style: "width: 100%; height: 4px; background: #e2e8f0; border-radius: 2px; overflow: hidden;",
                                div { style: "width: {progress_pct:.0}%; height: 100%; background: var(--carbon-dark); border-radius: 2px; transition: width 0.25s ease;" }
                            }
                        }

                        // Seção de Itens da Lista (Agrupados por Categoria em Inset-Groups)
                        if pending_items.is_empty() && purchased_items.is_empty() {
                            div { style: "text-align: center; padding: 60px 20px; color: var(--text-muted);",
                                p { style: "font-size: 1.05rem; font-weight: 600; color: var(--text-secondary); margin-bottom: 6px;", "Sua lista está vazia" }
                                p { style: "font-size: 0.85rem;", "Toque no botão '+' abaixo para adicionar produtos." }
                            }
                        } else {
                            // Categorias com produtos pendentes
                            for cat in CategoryId::all().iter().filter(|c| pending_items.iter().any(|i| i.category_id == **c)) {
                                div { key: "{cat.as_str()}", class: "inset-group",
                                    // Cabeçalho da Seção de Mercado
                                    div { class: "aisle-header",
                                        span { class: "aisle-dot", style: "background-color: {cat.color_hex()};" }
                                        span { "{cat.display_name()}" }
                                    }

                                    // Linhas de itens
                                    for item in pending_items.iter().filter(|i| i.category_id == *cat) {
                                        div { key: "{item.id}", class: "minimal-item-row",
                                            input {
                                                class: "minimal-checkbox",
                                                r#type: "checkbox",
                                                checked: false,
                                                onchange: {
                                                    let item_id = item.id;
                                                    let price = item.actual_price.or(item.estimated_price);
                                                    move |_| handle_toggle_item(item_id, false, price)
                                                }
                                            }

                                            div { class: "item-info",
                                                div { class: "item-name", "{item.name}" }
                                                div { class: "item-meta",
                                                    span { class: "item-qty-tag", "{item.quantity} {item.unit.as_str()}" }
                                                    if let Some(adder) = &item.added_by_user_name {
                                                        span { "por {adder}" }
                                                    }
                                                }
                                            }

                                            if let Some(p) = item.actual_price.or(item.estimated_price) {
                                                div { class: "item-price", "R$ {p:.2}" }
                                            }

                                            button {
                                                class: "item-delete-btn",
                                                title: "Remover",
                                                onclick: {
                                                    let item_id = item.id;
                                                    move |_| handle_delete_item(item_id)
                                                },
                                                "✕"
                                            }
                                        }
                                    }
                                }
                            }

                            // Itens Concluídos (No Carrinho)
                            if !purchased_items.is_empty() {
                                div { style: "margin-top: 20px;",
                                    div { style: "font-size: 0.72rem; font-weight: 700; letter-spacing: 0.08em; text-transform: uppercase; color: var(--text-muted); margin-bottom: 8px; padding-left: 4px;",
                                        "CONCLUÍDOS ({purchased_items.len()})"
                                    }

                                    div { class: "inset-group",
                                        for item in purchased_items.iter() {
                                            div { key: "{item.id}", class: "minimal-item-row is-purchased",
                                                input {
                                                    class: "minimal-checkbox",
                                                    r#type: "checkbox",
                                                    checked: true,
                                                    onchange: {
                                                        let item_id = item.id;
                                                        let price = item.actual_price.or(item.estimated_price);
                                                        move |_| handle_toggle_item(item_id, true, price)
                                                    }
                                                }

                                                div { class: "item-info",
                                                    div { class: "item-name", "{item.name}" }
                                                    div { class: "item-meta",
                                                        span { class: "item-qty-tag", "{item.quantity} {item.unit.as_str()}" }
                                                        if let Some(buyer) = &item.purchased_by_user_name {
                                                            span { "marcado por {buyer}" }
                                                        }
                                                    }
                                                }

                                                if let Some(p) = item.actual_price.or(item.estimated_price) {
                                                    div { class: "item-price", "R$ {p:.2}" }
                                                }

                                                button {
                                                    class: "item-delete-btn",
                                                    title: "Remover",
                                                    onclick: {
                                                        let item_id = item.id;
                                                        move |_| handle_delete_item(item_id)
                                                    },
                                                    "✕"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Barra Flutuante de Resumo e Ação (A exata pílula da sua imagem anexada!)
                        div { class: "floating-summary-pill",
                            div { class: "floating-summary-text",
                                span { "{d.totals.total_items} itens" }
                                span { "•" }
                                span { "Total: " }
                                strong { "R$ {d.totals.total_planned:.2}" }
                            }
                            button {
                                class: "floating-add-btn",
                                title: "Adicionar produto",
                                onclick: move |_| show_add_drawer.set(true),
                                "+"
                            }
                        }

                        // Bottom Sheet / Gaveta para Adicionar Item
                        if *show_add_drawer.read() {
                            div {
                                class: "drawer-overlay",
                                onclick: move |_| show_add_drawer.set(false),
                                div {
                                    class: "drawer-content",
                                    onclick: move |e| e.stop_propagation(),

                                    div { class: "drawer-handle" }

                                    div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                                        h3 { style: "font-size: 1.15rem; font-weight: 700; color: var(--text-primary);", "Adicionar à Lista" }
                                        button {
                                            class: "btn-ghost",
                                            onclick: move |_| show_add_drawer.set(false),
                                            "✕"
                                        }
                                    }

                                    div { class: "form-group",
                                        input {
                                            class: "form-input",
                                            placeholder: "Nome do produto (ex: Leite, Maçã gala, Picanha...)",
                                            value: "{new_item_name}",
                                            oninput: move |e| new_item_name.set(e.value())
                                        }
                                    }

                                    // Preview em tempo real da categoria
                                    if !new_item_name.read().trim().is_empty() {
                                        div { style: "display: flex; align-items: center; gap: 6px; margin-bottom: 14px; font-size: 0.82rem;",
                                            span { style: "color: var(--text-secondary);", "Corredor sugerido:" }
                                            span {
                                                class: "badge",
                                                style: "background-color: {preview_category.color_hex()}; color: #ffffff;",
                                                "{preview_category.display_name()}"
                                            }
                                        }
                                    }

                                    div { style: "display: flex; gap: 8px; margin-bottom: 20px;",
                                        div { style: "flex: 1;",
                                            input {
                                                class: "form-input",
                                                r#type: "number",
                                                placeholder: "Qtd",
                                                value: "{new_item_qty}",
                                                oninput: move |e| new_item_qty.set(e.value())
                                            }
                                        }
                                        div { style: "flex: 1.2;",
                                            select {
                                                class: "form-input",
                                                onchange: move |e| {
                                                    new_item_unit.set(QuantityUnit::from_unit_str(&e.value()));
                                                },
                                                option { value: "un", "un" }
                                                option { value: "kg", "kg" }
                                                option { value: "g", "g" }
                                                option { value: "l", "l" }
                                                option { value: "pct", "pct" }
                                                option { value: "cx", "cx" }
                                                option { value: "dz", "dz" }
                                            }
                                        }
                                        div { style: "flex: 1.5;",
                                            input {
                                                class: "form-input",
                                                placeholder: "R$ Estimado",
                                                value: "{new_item_price}",
                                                oninput: move |e| new_item_price.set(e.value())
                                            }
                                        }
                                    }

                                    button {
                                        class: "btn btn-primary",
                                        style: "width: 100%; padding: 12px; font-size: 0.95rem;",
                                        onclick: handle_add_item,
                                        "Adicionar Produto"
                                    }
                                }
                            }
                        }
                    }
                }
            } else if *is_loading.read() {
                div { style: "text-align: center; padding: 60px; color: var(--text-muted);", "Carregando itens..." }
            }

            // Modal: Mais Comprados / Sugestões Inteligentes
            if *show_suggestions_modal.read() {
                div { class: "modal-overlay",
                    div { class: "modal-content", style: "max-height: 80vh; overflow-y: auto;",
                        div { style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;",
                            h3 { style: "font-size: 1.25rem;", "✨ Itens Mais Comprados" }
                            button {
                                class: "btn-icon",
                                style: "color: #64748b; font-size: 1.2rem;",
                                onclick: move |_| show_suggestions_modal.set(false),
                                "✕"
                            }
                        }

                        if *loading_suggestions.read() {
                            p { style: "color: #64748b; text-align: center; padding: 20px;", "Carregando histórico de compras..." }
                        } else if suggestions.read().is_empty() {
                            p { style: "color: #64748b; text-align: center; padding: 20px;",
                                "Nenhum histórico registrado ainda. Marque itens como comprados para gerar sugestões automáticas!"
                            }
                        } else {
                            p { style: "font-size: 0.85rem; color: #64748b; margin-bottom: 12px;",
                                "Toque em qualquer item para adicionar rapidamente à sua lista:"
                            }

                            for item in suggestions.read().iter() {
                                {
                                    let name = item.name.clone();
                                    let cat = item.category_id;
                                    let unit = item.last_unit;
                                    let price = item.last_price;

                                    rsx! {
                                        div {
                                            key: "{item.name}",
                                            class: "item-card",
                                            style: "padding: 10px 14px; margin-bottom: 8px; justify-content: space-between;",

                                            div {
                                                div { style: "font-weight: 600; text-transform: capitalize;", "{item.name}" }
                                                div { style: "font-size: 0.8rem; color: #64748b; display: flex; gap: 8px; margin-top: 2px;",
                                                    span { "🔁 Comprado {item.purchase_count}x" }
                                                    if let Some(p) = item.last_price {
                                                        span { "• Último: R$ {p:.2}" }
                                                    }
                                                }
                                            }

                                            button {
                                                class: "btn btn-primary",
                                                style: "padding: 6px 12px; font-size: 0.85rem; min-height: 36px;",
                                                onclick: move |_| {
                                                    add_suggested_item(name.clone(), cat, unit, price);
                                                },
                                                "➕ Adicionar"
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
