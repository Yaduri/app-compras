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

    // Modo Supermercado (Alto Contraste e Fontes Maiores)
    let mut is_market_mode = use_signal(|| false);

    // Modal de Sugestões / Mais Comprados
    let mut show_suggestions_modal = use_signal(|| false);
    let mut suggestions = use_signal(Vec::<FrequentItemResponse>::new);
    let mut loading_suggestions = use_signal(|| false);

    // Notificação em tempo real (ex: "Maria marcou Leite no carrinho")
    let mut real_time_alert = use_signal(|| None::<String>);

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

    let market_mode_class = if *is_market_mode.read() { "market-mode" } else { "" };

    rsx! {
        div { class: "main-content {market_mode_class}", style: "min-height: 100vh; display: flex; flex-direction: column;",
            // Barra Superior de Controle
            div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 14px; gap: 8px; flex-wrap: wrap;",
                button {
                    class: "btn btn-secondary",
                    style: "padding: 8px 14px; min-height: 40px;",
                    onclick: move |_| active_list_id.set(None),
                    "← Voltar"
                }

                div { style: "display: flex; align-items: center; gap: 8px;",
                    // Botão Alternador do Modo Supermercado
                    button {
                        class: if *is_market_mode.read() { "btn btn-success" } else { "btn btn-secondary" },
                        style: "padding: 8px 14px; min-height: 40px; font-weight: 700;",
                        onclick: move |_| is_market_mode.toggle(),
                        if *is_market_mode.read() { "🛒 Modo Mercado Ativo" } else { "🛒 Modo Mercado" }
                    }

                    if let Some(d) = detail.read().as_ref() {
                        span {
                            class: "badge badge-code",
                            style: "font-size: 0.95rem; padding: 6px 12px;",
                            title: "Código para convidar participantes",
                            "🔑 {d.list.invite_code}"
                        }
                    }
                }
            }

            // Alerta de Notificação em Tempo Real (WebSocket)
            if let Some(msg) = real_time_alert.read().as_ref() {
                div {
                    class: "alert",
                    style: "background-color: #f0fdf4; color: #166534; border: 1px solid #86efac; display: flex; justify-content: space-between; align-items: center; animation: pulse 1s;",
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
                        // Card do Cabeçalho e Totais Parciais em Tempo Real
                        div { class: "card", style: "border-top: 4px solid #10b981;",
                            div { style: "display: flex; justify-content: space-between; align-items: flex-start;",
                                div {
                                    h2 { style: "font-size: 1.5rem; font-weight: 700; color: #0f172a; margin-bottom: 4px;", "{d.list.name}" }
                                    if let Some(desc) = &d.list.description {
                                        p { style: "color: #64748b; font-size: 0.9rem; margin-bottom: 10px;", "{desc}" }
                                    }
                                }
                                div { style: "display: flex; gap: 4px;",
                                    button {
                                        class: "btn btn-secondary",
                                        style: "padding: 6px 12px; font-size: 0.85rem; min-height: 36px;",
                                        onclick: open_suggestions,
                                        "✨ Mais Comprados"
                                    }
                                }
                            }

                            // Barra de Progresso
                            div { class: "progress-container", style: "height: 8px; margin: 12px 0 10px 0;",
                                div { class: "progress-bar", style: "width: {progress_pct:.0}%;" }
                            }

                            // Totais Orçamentários (Total Planejado vs Total no Carrinho)
                            div { style: "display: flex; justify-content: space-between; align-items: center; margin-top: 8px; font-size: 0.95rem; flex-wrap: wrap; gap: 8px;",
                                div {
                                    span { style: "color: #64748b;", "Total Planejado: " }
                                    strong { "R$ {d.totals.total_planned:.2}" }
                                }
                                div {
                                    span { style: "color: #059669; font-weight: 600;", "No Carrinho: " }
                                    strong { style: "color: #059669; font-size: 1.15rem; font-weight: 800;", "R$ {d.totals.total_cart:.2}" }
                                }
                                div {
                                    span { style: "color: #059669; font-weight: 600; background-color: #d1fae5; padding: 4px 8px; border-radius: 6px;", "{d.totals.purchased_items}/{d.totals.total_items} itens ({progress_pct:.0}%)" }
                                }
                            }
                        }

                        // Formulário de Adição Rápida de Produto com Autocategorização
                        if !*is_market_mode.read() {
                            div { class: "card", style: "background-color: #ffffff; border: 1px solid #e2e8f0;",
                                h3 { style: "font-size: 1rem; font-weight: 700; margin-bottom: 12px; color: #1e293b;", "⚡ Adição Rápida com Autocategorização" }

                                div { style: "display: flex; gap: 8px; margin-bottom: 8px; flex-wrap: wrap;",
                                    div { style: "flex: 3; min-width: 200px;",
                                        input {
                                            class: "form-input",
                                            placeholder: "Ex: Leite, Maçã gala, Picanha, Detergente...",
                                            value: "{new_item_name}",
                                            oninput: move |e| new_item_name.set(e.value())
                                        }
                                    }
                                    div { style: "flex: 1; min-width: 70px;",
                                        input {
                                            class: "form-input",
                                            r#type: "number",
                                            placeholder: "Qtd",
                                            value: "{new_item_qty}",
                                            oninput: move |e| new_item_qty.set(e.value())
                                        }
                                    }
                                    div { style: "flex: 1; min-width: 80px;",
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
                                    div { style: "flex: 1; min-width: 90px;",
                                        input {
                                            class: "form-input",
                                            placeholder: "R$ Estimado",
                                            value: "{new_item_price}",
                                            oninput: move |e| new_item_price.set(e.value())
                                        }
                                    }
                                }

                                // Preview em tempo real da categoria identificada
                                if !new_item_name.read().trim().is_empty() {
                                    div { style: "display: flex; align-items: center; gap: 6px; margin-bottom: 12px; font-size: 0.85rem;",
                                        span { style: "color: #64748b;", "Categoria Identificada:" }
                                        span {
                                            class: "badge",
                                            style: "background-color: {preview_category.color_hex()}; color: #ffffff;",
                                            "{preview_category.display_name()}"
                                        }
                                    }
                                }

                                button {
                                    class: "btn btn-primary",
                                    style: "width: 100%; font-size: 1rem;",
                                    onclick: handle_add_item,
                                    "➕ Adicionar à Lista"
                                }
                            }
                        }

                        // Seção de Itens Pendentes (Agrupados por Seção do Mercado)
                        if pending_items.is_empty() && purchased_items.is_empty() {
                            div { style: "text-align: center; padding: 40px; color: #64748b;",
                                "Sua lista está vazia. Adicione produtos acima para iniciar as compras!"
                            }
                        } else {
                            // Itera sobre as categorias com produtos pendentes
                            for cat in CategoryId::all().iter().filter(|c| pending_items.iter().any(|i| i.category_id == **c)) {
                                div { key: "{cat.as_str()}", style: "margin-bottom: 22px;",
                                    // Cabeçalho da Seção de Mercado
                                    div { style: "display: flex; align-items: center; gap: 8px; margin-bottom: 10px;",
                                        span {
                                            style: "width: 14px; height: 14px; border-radius: 50%; background-color: {cat.color_hex()}; display: inline-block;"
                                        }
                                        h4 { style: "font-size: 1.15rem; font-weight: 700;",
                                            "{cat.display_name()}"
                                        }
                                    }

                                    // Lista de Itens da Categoria
                                    for item in pending_items.iter().filter(|i| i.category_id == *cat) {
                                        div {
                                            key: "{item.id}",
                                            class: "item-card",

                                            // Checkbox Grande de Supermercado
                                            input {
                                                class: "market-checkbox",
                                                r#type: "checkbox",
                                                checked: item.purchased,
                                                onchange: {
                                                    let item_id = item.id;
                                                    let is_purchased = item.purchased;
                                                    let price = item.actual_price.or(item.estimated_price);
                                                    move |_| handle_toggle_item(item_id, is_purchased, price)
                                                }
                                            }

                                            div { class: "item-details",
                                                div { style: "display: flex; justify-content: space-between; align-items: baseline;",
                                                    span { class: "item-name", "{item.name}" }
                                                    if let Some(p) = item.actual_price.or(item.estimated_price) {
                                                        span { style: "font-weight: 700; font-size: 1.05rem; margin-left: 8px;",
                                                            "R$ {p:.2}"
                                                        }
                                                    }
                                                }
                                                div { class: "item-meta",
                                                    span { class: "badge", style: "background-color: #e2e8f0; color: #334155;",
                                                        "{item.quantity} {item.unit.as_str()}"
                                                    }
                                                    if let Some(adder) = &item.added_by_user_name {
                                                        span { style: "color: #94a3b8;", "• Adicionado por {adder}" }
                                                    }
                                                }
                                            }

                                            // Botão de Excluir Item
                                            button {
                                                class: "btn-icon",
                                                style: "color: #94a3b8; font-size: 1.1rem; padding: 6px 10px;",
                                                title: "Excluir item",
                                                onclick: {
                                                    let item_id = item.id;
                                                    move |_| handle_delete_item(item_id)
                                                },
                                                "🗑️"
                                            }
                                        }
                                    }
                                }
                            }

                            // Seção: Itens no Carrinho (Comprados) - Vão automaticamente para o final
                            if !purchased_items.is_empty() {
                                div { style: "margin-top: 30px; border-top: 2px dashed #cbd5e1; padding-top: 20px;",
                                    h4 { style: "font-size: 1.15rem; color: #059669; font-weight: 700; margin-bottom: 12px; display: flex; align-items: center; gap: 6px;",
                                        "✅ No Carrinho ({purchased_items.len()})"
                                    }

                                    for item in purchased_items.iter() {
                                        div {
                                            key: "{item.id}",
                                            class: "item-card purchased",

                                            input {
                                                class: "market-checkbox",
                                                r#type: "checkbox",
                                                checked: true,
                                                onchange: {
                                                    let item_id = item.id;
                                                    let price = item.actual_price.or(item.estimated_price);
                                                    move |_| handle_toggle_item(item_id, true, price)
                                                }
                                            }

                                            div { class: "item-details",
                                                div { style: "display: flex; justify-content: space-between; align-items: baseline;",
                                                    span { class: "item-name", "{item.name}" }
                                                    if let Some(p) = item.actual_price.or(item.estimated_price) {
                                                        span { style: "font-weight: 700; color: #059669; font-size: 1.05rem; margin-left: 8px;",
                                                            "R$ {p:.2}"
                                                        }
                                                    }
                                                }
                                                div { class: "item-meta",
                                                    span { class: "badge", style: "background-color: #d1fae5; color: #065f46; font-weight: 600;",
                                                        "{item.quantity} {item.unit.as_str()}"
                                                    }
                                                    if let Some(buyer) = &item.purchased_by_user_name {
                                                        span { style: "color: #059669; font-weight: 500;", "• Marcado por {buyer}" }
                                                    }
                                                }
                                            }

                                            button {
                                                class: "btn-icon",
                                                style: "color: #94a3b8; font-size: 1.1rem; padding: 6px 10px;",
                                                onclick: {
                                                    let item_id = item.id;
                                                    move |_| handle_delete_item(item_id)
                                                },
                                                "🗑️"
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // Barra Fixa de Rodapé no Modo Supermercado
                        if *is_market_mode.read() {
                            div { class: "sticky-cart-summary",
                                div {
                                    div { style: "font-size: 0.85rem; opacity: 0.9;", "Subtotal no Carrinho:" }
                                    div { style: "font-size: 1.4rem; font-weight: 800;", "R$ {d.totals.total_cart:.2}" }
                                }
                                div { style: "text-align: right;",
                                    div { style: "font-size: 0.85rem; opacity: 0.9;", "Restam comprar:" }
                                    div { style: "font-size: 1.2rem; font-weight: 700;", "{pending_items.len()} itens" }
                                }
                            }
                        }
                    }
                }
            } else if *is_loading.read() {
                div { style: "text-align: center; padding: 60px; color: #64748b;", "Carregando itens da lista de compras..." }
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
