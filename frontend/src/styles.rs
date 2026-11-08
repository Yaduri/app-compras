pub const APP_CSS: &str = r#"
/* ==============================================================================
   Design System: minimalista e neutro, pensado para uso com uma mão no mercado.
   Todos os tokens usados nas telas precisam estar definidos aqui.
   ============================================================================== */

*, *::before, *::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    -webkit-tap-highlight-color: transparent;
}

:root {
    --bg-canvas: #f8fafc;
    --surface: #ffffff;
    --bg-card: #ffffff;
    --surface-subtle: #f1f5f9;
    --surface-hover: #f8fafc;
    --surface-pressed: #eef2f6;
    --border: #e2e8f0;
    --border-light: #e2e8f0;
    --border-hairline: #f1f5f9;
    /* Contraste AA: secundário 7.2:1 e "muted" 4.7:1 sobre o fundo */
    --text-primary: #09090b;
    --text-secondary: #475569;
    --text-muted: #64748b;
    --carbon-dark: #0f172a;
    --carbon-hover: #1e293b;
    --accent: #0f172a;
    --accent-emerald: #10b981;
    --accent-emerald-subtle: #ecfdf5;
    /* Vermelho de ação: 4.8:1 sobre branco */
    --danger: #dc2626;
    --danger-strong: #b91c1c;
    --danger-subtle: #fef2f2;
    --danger-border: #fecaca;
    --radius-pill: 9999px;
    --radius-btn: 9999px;
    --radius-card: 16px;
    --radius-sm: 8px;
    --tap: 44px;
    --font-sans: Roboto, system-ui, -apple-system, "Segoe UI", "Helvetica Neue", Arial, sans-serif;
}

html, body {
    width: 100%;
    height: 100%;
    background-color: var(--bg-canvas);
    color: var(--text-primary);
    font-family: var(--font-sans);
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    line-height: 1.5;
    caret-color: var(--carbon-dark);
}

/* Controles herdam a fonte do app (sem Arial do navegador) */
button, input, select, textarea {
    font: inherit;
    color: inherit;
}

::selection {
    background: rgba(15, 23, 42, 0.14);
    color: var(--text-primary);
}

/* Foco visível só para teclado/leitor de tela; toque continua limpo */
:focus { outline: none; }
:focus-visible {
    outline: 2px solid var(--carbon-dark);
    outline-offset: 2px;
}

h1, h2, h3 {
    font-weight: 700;
    letter-spacing: -0.02em;
    text-wrap: balance;
}

.visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
}

.icon {
    display: inline-block;
    flex-shrink: 0;
    vertical-align: middle;
}

/* Shell do App */
.app-container {
    width: 100%;
    min-height: 100vh;
    background-color: var(--bg-canvas);
    display: flex;
    flex-direction: column;
    margin: 0 auto;
    position: relative;
}

/* Barra superior */
.navbar {
    background: rgba(248, 250, 252, 0.92);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--border);
    padding: 6px 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    position: sticky;
    top: 0;
    z-index: 50;
}

.navbar-brand {
    font-size: 1.05rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
}

.brand-badge {
    background: var(--carbon-dark);
    color: #ffffff;
    width: 28px;
    height: 28px;
    border-radius: 8px;
    display: flex;
    align-items: center;
    justify-content: center;
}

.live-indicator {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 0.72rem;
    font-weight: 600;
    color: var(--text-secondary);
    background: #ffffff;
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
}

.live-dot {
    width: 6px;
    height: 6px;
    background-color: var(--accent-emerald);
    border-radius: 50%;
}

.nav-actions {
    display: flex;
    align-items: center;
    gap: 4px;
}

.user-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 4px 6px;
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--text-primary);
    max-width: 40vw;
}

.user-chip span {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.user-avatar-circle {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--carbon-dark);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 700;
    flex-shrink: 0;
}

.btn-logout {
    min-height: var(--tap);
    min-width: var(--tap);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    color: var(--text-secondary);
    border: none;
    border-radius: var(--radius-pill);
    padding: 0 12px;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
}

.btn-logout:active {
    background: var(--danger-subtle);
    color: var(--danger-strong);
}

/* Conteúdo principal */
.main-content {
    flex: 1;
    width: 100%;
    max-width: 600px;
    margin: 0 auto;
    padding: 20px 16px 32px;
    display: flex;
    flex-direction: column;
}

/* Telas com a pílula fixa embaixo reservam espaço para ela */
.main-content.has-pill {
    padding-bottom: 120px;
}

.page-header {
    display: flex;
    flex-direction: column;
    gap: 14px;
    margin-bottom: 20px;
}

.page-title {
    font-size: 1.5rem;
    line-height: 1.2;
    color: var(--text-primary);
}

.page-subtitle {
    color: var(--text-secondary);
    font-size: 0.88rem;
    margin-top: 4px;
}

.page-actions {
    display: flex;
    gap: 8px;
}

.page-actions .btn {
    flex: 1;
}

/* Cartão base */
.card {
    background: var(--bg-card);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    padding: 20px;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.04);
}

/* Cartão de lista no painel (inteiro é tocável) */
.list-card {
    display: block;
    width: 100%;
    text-align: left;
    cursor: pointer;
    padding: 18px;
    transition: background-color 0.15s ease, border-color 0.15s ease;
}

.list-card:active {
    background: var(--surface-pressed);
}

.list-card-head {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 12px;
}

.list-card-title {
    font-size: 1.12rem;
    line-height: 1.3;
    color: var(--text-primary);
    word-break: break-word;
}

.list-card-desc {
    font-size: 0.85rem;
    color: var(--text-secondary);
    margin-top: 2px;
    line-height: 1.35;
}

.list-card-foot {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 12px;
    font-size: 0.85rem;
    color: var(--text-secondary);
}

.list-card-total strong {
    color: var(--text-primary);
    font-size: 1rem;
    font-variant-numeric: tabular-nums;
}

/* Barra de progresso */
.progress {
    width: 100%;
    height: 4px;
    background: var(--border);
    border-radius: 2px;
    overflow: hidden;
    margin: 14px 0 10px;
}

.progress-fill {
    height: 100%;
    background: var(--carbon-dark);
    border-radius: 2px;
    transition: width 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

/* Barra de ferramentas da lista */
.list-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 8px;
}

.toolbar-actions {
    display: flex;
    align-items: center;
    gap: 8px;
}

.list-header {
    margin: 8px 0 20px;
}

.list-title {
    font-size: 1.6rem;
    line-height: 1.2;
    color: var(--text-primary);
    word-break: break-word;
}

.list-desc {
    color: var(--text-secondary);
    font-size: 0.88rem;
    margin-top: 4px;
}

.list-progress-label {
    display: flex;
    justify-content: space-between;
    font-size: 0.82rem;
    color: var(--text-secondary);
    margin-top: 14px;
}

.list-progress-label + .progress {
    margin-top: 6px;
}

/* Grupo de itens de um corredor */
.inset-group {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
    margin-bottom: 16px;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.04);
}

.aisle-header {
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
    padding: 12px 16px 8px;
    display: flex;
    align-items: center;
    gap: 8px;
    border-bottom: 1px solid var(--border-hairline);
}

.aisle-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
}

.section-label {
    font-size: 0.75rem;
    font-weight: 700;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--text-secondary);
    margin: 8px 0 8px 4px;
}

/* Linha de item: a linha inteira marca/desmarca */
.minimal-item-row {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 60px;
    padding: 8px 4px 8px 16px;
    background: var(--surface);
    border-bottom: 1px solid var(--border-hairline);
    cursor: pointer;
    user-select: none;
    transition: background-color 0.12s ease;
}

.minimal-item-row:last-child {
    border-bottom: none;
}

.minimal-item-row:active {
    background-color: var(--surface-pressed);
}

/* Concluídos: riscado e cor secundária, sem perder contraste */
.minimal-item-row.is-purchased .item-name {
    text-decoration: line-through;
    text-decoration-color: var(--text-muted);
    color: var(--text-secondary);
}

.minimal-item-row.is-purchased .item-price {
    color: var(--text-secondary);
    font-weight: 500;
}

/* Marcador circular (visual; o alvo de toque é a linha) */
.check-circle {
    width: 24px;
    height: 24px;
    border: 2px solid #94a3b8;
    border-radius: 50%;
    display: grid;
    place-content: center;
    flex-shrink: 0;
    background: #ffffff;
    color: #ffffff;
    transition: background-color 0.18s cubic-bezier(0.16, 1, 0.3, 1), border-color 0.18s ease, transform 0.18s cubic-bezier(0.16, 1, 0.3, 1);
}

.minimal-item-row:active .check-circle {
    transform: scale(0.88);
}

.check-circle.checked {
    background-color: var(--carbon-dark);
    border-color: var(--carbon-dark);
}

.item-info {
    flex: 1;
    min-width: 0;
}

.item-name {
    font-size: 1rem;
    font-weight: 600;
    color: var(--text-primary);
    line-height: 1.3;
    word-break: break-word;
}

.item-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 8px;
    margin-top: 3px;
    font-size: 0.8rem;
    color: var(--text-secondary);
}

.item-qty-tag {
    font-size: 0.78rem;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--surface-subtle);
    padding: 1px 6px;
    border-radius: 4px;
    font-variant-numeric: tabular-nums;
}

.item-price {
    font-size: 0.9rem;
    font-weight: 500;
    color: var(--text-secondary);
    white-space: nowrap;
    font-variant-numeric: tabular-nums;
}

/* Botão de remover: 44x44, discreto mas legível */
.item-delete-btn {
    width: var(--tap);
    height: var(--tap);
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
}

.item-delete-btn:active {
    background: var(--danger-subtle);
    color: var(--danger-strong);
}

.empty-state {
    text-align: center;
    padding: 56px 20px;
}

.empty-state h2 {
    font-size: 1.1rem;
    color: var(--text-primary);
    margin-bottom: 6px;
}

.empty-state p {
    font-size: 0.9rem;
    color: var(--text-secondary);
}

.loading-state {
    text-align: center;
    padding: 56px 20px;
    color: var(--text-secondary);
    font-size: 0.9rem;
}

.danger-zone {
    margin-top: 32px;
    text-align: center;
}

/* Pílula fixa inferior: progresso + total no carrinho + adicionar */
.floating-summary-pill {
    position: fixed;
    bottom: calc(20px + env(safe-area-inset-bottom, 0px));
    left: 50%;
    transform: translateX(-50%);
    width: calc(100% - 32px);
    max-width: 440px;
    min-height: 62px;
    background: var(--carbon-dark);
    border-radius: var(--radius-pill);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 8px 8px 22px;
    z-index: 100;
    box-shadow: 0 12px 28px -6px rgba(15, 23, 42, 0.4), 0 4px 10px -4px rgba(15, 23, 42, 0.25);
}

.pill-summary {
    display: flex;
    align-items: center;
    gap: 14px;
    min-width: 0;
    color: #f8fafc;
    user-select: none;
}

.pill-count {
    font-size: 0.95rem;
    font-weight: 700;
    font-variant-numeric: tabular-nums;
    padding-right: 14px;
    border-right: 1px solid rgba(255, 255, 255, 0.18);
}

.pill-totals {
    display: flex;
    flex-direction: column;
    line-height: 1.2;
    min-width: 0;
}

.pill-totals strong {
    font-size: 1.05rem;
    font-weight: 700;
    color: #ffffff;
    font-variant-numeric: tabular-nums;
}

.pill-sub {
    font-size: 0.75rem;
    color: #cbd5e1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.floating-add-btn {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: #ffffff;
    color: var(--carbon-dark);
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
    transition: transform 0.15s ease, background-color 0.15s ease;
    flex-shrink: 0;
}

.floating-add-btn:active {
    transform: scale(0.92);
    background-color: #e2e8f0;
}

/* Avisos temporários (acima da pílula, sem empurrar a lista) */
.toast-stack {
    position: fixed;
    left: 50%;
    transform: translateX(-50%);
    bottom: calc(96px + env(safe-area-inset-bottom, 0px));
    width: calc(100% - 32px);
    max-width: 440px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    z-index: 120;
    pointer-events: none;
}

.toast {
    pointer-events: auto;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 48px;
    padding: 6px 6px 6px 16px;
    background: #1e293b;
    color: #f8fafc;
    border-radius: 14px;
    font-size: 0.88rem;
    box-shadow: 0 8px 20px -6px rgba(15, 23, 42, 0.45);
    animation: toastIn 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}

.toast-text {
    padding: 6px 0;
}

.toast-action {
    min-height: 40px;
    padding: 0 14px;
    background: transparent;
    border: none;
    border-radius: 10px;
    color: #ffffff;
    font-weight: 700;
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 3px;
}

.toast-action:active {
    background: rgba(255, 255, 255, 0.12);
}

/* Gaveta inferior (adicionar item, sugestões) */
.drawer-overlay {
    position: fixed;
    inset: 0;
    background: rgba(15, 23, 42, 0.45);
    z-index: 200;
    display: flex;
    align-items: flex-end;
    justify-content: center;
    animation: fadeIn 0.15s ease-out;
}

.drawer-content {
    background: var(--surface);
    width: 100%;
    max-width: 560px;
    max-height: 85vh;
    overflow-y: auto;
    border-top-left-radius: 24px;
    border-top-right-radius: 24px;
    padding: 12px 20px calc(24px + env(safe-area-inset-bottom, 0px));
    box-shadow: 0 -10px 40px rgba(15, 23, 42, 0.18);
    animation: slideUp 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}

.drawer-handle {
    width: 36px;
    height: 4px;
    background: #cbd5e1;
    border-radius: 2px;
    margin: 0 auto 12px;
}

.sheet-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 14px;
}

.sheet-title {
    font-size: 1.15rem;
    color: var(--text-primary);
}

.sheet-hint {
    font-size: 0.85rem;
    color: var(--text-secondary);
    margin-bottom: 12px;
}

/* Botão de fechar/ícone padrão (44x44) */
.icon-btn {
    width: var(--tap);
    height: var(--tap);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 50%;
    color: var(--text-secondary);
    cursor: pointer;
    flex-shrink: 0;
}

.icon-btn:active {
    background: var(--surface-subtle);
    color: var(--text-primary);
}

/* Botões */
.btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    min-height: var(--tap);
    padding: 0 18px;
    font-size: 0.92rem;
    font-weight: 600;
    border-radius: var(--radius-btn);
    border: 1px solid transparent;
    cursor: pointer;
    transition: background-color 0.15s ease, border-color 0.15s ease, transform 0.1s ease;
    text-decoration: none;
    white-space: nowrap;
}

.btn:disabled {
    opacity: 0.6;
    cursor: default;
}

.btn:active:not(:disabled) {
    transform: scale(0.98);
}

.btn-sm {
    padding: 0 14px;
    font-size: 0.85rem;
}

.btn-block {
    width: 100%;
}

.btn-primary {
    background-color: var(--carbon-dark);
    color: #ffffff;
}

.btn-primary:active:not(:disabled) {
    background-color: var(--carbon-hover);
}

.btn-secondary {
    background-color: #ffffff;
    color: var(--text-primary);
    border-color: var(--border);
}

.btn-secondary:active:not(:disabled) {
    background-color: var(--surface-subtle);
}

.btn-danger {
    background-color: var(--danger);
    color: #ffffff;
}

.btn-danger:active:not(:disabled) {
    background-color: var(--danger-strong);
}

/* Ação destrutiva discreta (texto) */
.btn-danger-text {
    background: transparent;
    color: var(--danger-strong);
    border: none;
}

.btn-danger-text:active:not(:disabled) {
    background: var(--danger-subtle);
}

.btn-google {
    width: 100%;
    min-height: var(--tap);
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 10px;
    background: #ffffff;
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-btn);
    padding: 0 16px;
    font-size: 0.92rem;
    font-weight: 600;
    cursor: pointer;
    box-shadow: 0 1px 2px rgba(15, 23, 42, 0.05);
}

.btn-google:active:not(:disabled) {
    background: var(--surface-subtle);
}

.btn-code {
    font-family: ui-monospace, "Roboto Mono", Menlo, Consolas, monospace;
    letter-spacing: 0.04em;
}

/* Formulários */
.form-input {
    width: 100%;
    min-height: 48px;
    padding: 10px 14px;
    font-size: 1rem;
    border: 1px solid #cbd5e1;
    border-radius: 12px;
    background-color: #ffffff;
    color: var(--text-primary);
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
}

.form-input::placeholder {
    color: var(--text-muted);
}

.form-input:focus {
    border-color: var(--carbon-dark);
    box-shadow: 0 0 0 1px var(--carbon-dark);
}

.form-label {
    display: block;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 6px;
}

.form-group {
    margin-bottom: 16px;
}

.form-row {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
}

.form-error {
    font-size: 0.85rem;
    color: var(--danger-strong);
    margin: -6px 0 12px;
}

.code-input {
    text-transform: uppercase;
    font-family: ui-monospace, "Roboto Mono", Menlo, Consolas, monospace;
    font-size: 1.2rem;
    letter-spacing: 0.12em;
}

.category-preview {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: -4px 0 14px;
    font-size: 0.85rem;
    color: var(--text-secondary);
}

.category-chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 3px 10px;
    border-radius: var(--radius-pill);
    font-weight: 600;
    color: var(--text-primary);
}

/* Seletor Entrar / Criar conta */
.segmented-control {
    display: flex;
    background-color: var(--surface-subtle);
    padding: 3px;
    border-radius: 12px;
    margin-bottom: 20px;
}

.segmented-btn {
    flex: 1;
    min-height: 40px;
    padding: 0 14px;
    font-size: 0.9rem;
    font-weight: 600;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-radius: 10px;
    cursor: pointer;
    text-align: center;
}

.segmented-btn.active {
    background-color: #ffffff;
    color: var(--text-primary);
    box-shadow: 0 1px 3px rgba(15, 23, 42, 0.1);
}

.divider {
    display: flex;
    align-items: center;
    gap: 12px;
    margin: 18px 0;
    color: var(--text-muted);
    font-size: 0.85rem;
}

.divider::before,
.divider::after {
    content: "";
    flex: 1;
    height: 1px;
    background: var(--border);
}

/* Etiquetas */
.badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 8px;
    border-radius: 6px;
    font-size: 0.78rem;
    font-weight: 500;
    line-height: 1;
    white-space: nowrap;
    background: var(--surface-subtle);
    color: var(--text-secondary);
}

/* Alertas */
.alert {
    padding: 4px 4px 4px 16px;
    min-height: 48px;
    border-radius: 12px;
    font-size: 0.9rem;
    margin-bottom: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
}

.alert-error {
    background-color: var(--danger-subtle);
    color: #991b1b;
    border: 1px solid var(--danger-border);
}

.alert-success {
    background-color: #f0fdf4;
    color: #166534;
    border: 1px solid #bbf7d0;
}

.alert .icon-btn {
    color: inherit;
}

/* Lista de sugestões */
.suggestion-list {
    display: flex;
    flex-direction: column;
}

.item-card {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    min-height: 60px;
    padding: 8px 0;
    border-bottom: 1px solid var(--border-hairline);
}

.item-card:last-child {
    border-bottom: none;
}

.item-card-name {
    font-weight: 600;
    color: var(--text-primary);
}

.item-card-name::first-letter {
    text-transform: uppercase;
}

.item-card-meta {
    font-size: 0.82rem;
    color: var(--text-secondary);
    margin-top: 2px;
}

/* Modais de decisão (criar, entrar, excluir) */
.modal-overlay {
    position: fixed;
    inset: 0;
    background-color: rgba(15, 23, 42, 0.45);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 150;
    padding: 20px;
    animation: fadeIn 0.15s ease-out;
}

.modal-content {
    background-color: var(--surface);
    border-radius: 20px;
    width: 100%;
    max-width: 440px;
    padding: 24px;
    box-shadow: 0 20px 40px -10px rgba(15, 23, 42, 0.3);
    animation: scaleIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.modal-title {
    font-size: 1.2rem;
    color: var(--text-primary);
    margin-bottom: 8px;
}

.modal-text {
    color: var(--text-secondary);
    font-size: 0.92rem;
    line-height: 1.5;
    margin-bottom: 20px;
}

.modal-text strong {
    color: var(--text-primary);
}

.modal-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 8px;
}

/* Login */
.auth-screen {
    max-width: 420px;
    justify-content: center;
    min-height: calc(100vh - 20px);
}

.auth-hero {
    text-align: center;
    margin-bottom: 28px;
}

.auth-title {
    font-size: 1.75rem;
    line-height: 1.15;
    color: var(--text-primary);
}

.auth-subtitle {
    color: var(--text-secondary);
    font-size: 0.95rem;
    margin-top: 6px;
}

/* Animações */
@keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}

@keyframes slideUp {
    from { transform: translateY(100%); }
    to { transform: translateY(0); }
}

@keyframes scaleIn {
    from { opacity: 0; transform: scale(0.96); }
    to { opacity: 1; transform: scale(1); }
}

@keyframes toastIn {
    from { opacity: 0; transform: translateY(8px); }
    to { opacity: 1; transform: translateY(0); }
}

@media (prefers-reduced-motion: reduce) {
    *, *::before, *::after {
        animation-duration: 0.01ms !important;
        transition-duration: 0.01ms !important;
    }
}
"#;
