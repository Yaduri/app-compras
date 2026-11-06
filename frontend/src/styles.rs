pub const APP_CSS: &str = r#"
/* ==============================================================================
   Design System: Things 3 / Linear Mobile (Ultra Minimalista Editorial)
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
    --surface-subtle: #f1f5f9;
    --border: #e2e8f0;
    --border-hairline: #f1f5f9;
    --text-primary: #09090b;
    --text-secondary: #64748b;
    --text-muted: #94a3b8;
    --carbon-dark: #0f172a;
    --carbon-hover: #1e293b;
    --accent: #0f172a;
    --accent-emerald: #10b981;
    --accent-emerald-subtle: #ecfdf5;
    --radius-pill: 9999px;
    --radius-card: 16px;
    --radius-sm: 8px;
}

html, body {
    width: 100%;
    height: 100%;
    background-color: var(--bg-canvas);
    color: var(--text-primary);
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", "Inter", Roboto, Helvetica, Arial, sans-serif;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    line-height: 1.5;
}

/* Shell do App: Edge-to-Edge nativo */
.app-container {
    width: 100%;
    min-height: 100vh;
    background-color: var(--bg-canvas);
    display: flex;
    flex-direction: column;
    margin: 0 auto;
    position: relative;
    padding-bottom: 90px; /* espaço para o botão flutuante */
}

/* Header & Barra Superior Minimalista */
.navbar {
    background: rgba(248, 250, 252, 0.88);
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    border-bottom: 1px solid var(--border);
    padding: 14px 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: sticky;
    top: 0;
    z-index: 50;
}

.navbar h1 {
    font-size: 1.1rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    color: var(--text-primary);
    display: flex;
    align-items: center;
    gap: 8px;
}

.brand-badge {
    background: var(--carbon-dark);
    color: #ffffff;
    width: 28px;
    height: 28px;
    border-radius: 7px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.85rem;
    font-weight: 700;
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
    gap: 8px;
}

.user-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    background: #ffffff;
    padding: 4px 10px 4px 6px;
    border-radius: var(--radius-pill);
    border: 1px solid var(--border);
    font-size: 0.82rem;
    font-weight: 500;
    color: var(--text-primary);
}

.user-avatar-circle,
.user-avatar-sm {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    background: var(--carbon-dark);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.72rem;
    font-weight: 700;
}

.btn-logout,
.btn-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
    background: #ffffff;
    color: var(--text-secondary);
    border: 1px solid var(--border);
    border-radius: var(--radius-pill);
    padding: 4px 12px;
    font-size: 0.8rem;
    font-weight: 500;
    cursor: pointer;
    transition: all 0.15s ease;
    font-family: inherit;
    line-height: 1.4;
    outline: none;
}

.btn-logout:hover,
.btn-icon:hover {
    background: #fef2f2;
    border-color: #fecaca;
    color: var(--danger);
}

/* Conteúdo Principal */
.main-content {
    flex: 1;
    width: 100%;
    max-width: 600px;
    margin: 0 auto;
    padding: 24px 20px;
    display: flex;
    flex-direction: column;
}

/* Inset-Grouped Container (Estilo iOS / Things 3) */
.inset-group {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: var(--radius-card);
    overflow: hidden;
    margin-bottom: 20px;
    box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.02);
}

/* Linha de Item de Lista Minimalista */
.minimal-item-row {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px 18px;
    background: var(--surface);
    border-bottom: 1px solid var(--border-hairline);
    transition: background-color 0.15s ease, opacity 0.2s ease;
}

.minimal-item-row:last-child {
    border-bottom: none;
}

.minimal-item-row:hover {
    background-color: #fafbfc;
}

.minimal-item-row.is-purchased {
    opacity: 0.42;
}

.minimal-item-row.is-purchased .item-name {
    text-decoration: line-through;
    color: var(--text-secondary);
}

/* Checkbox Circular Minimalista (Things 3) */
.minimal-checkbox {
    appearance: none;
    -webkit-appearance: none;
    width: 22px;
    height: 22px;
    border: 1.5px solid #cbd5e1;
    border-radius: 50%;
    outline: none;
    cursor: pointer;
    display: grid;
    place-content: center;
    transition: all 0.18s cubic-bezier(0.16, 1, 0.3, 1);
    flex-shrink: 0;
    background: #ffffff;
}

.minimal-checkbox:hover {
    border-color: var(--carbon-dark);
}

.minimal-checkbox:checked {
    background-color: var(--carbon-dark);
    border-color: var(--carbon-dark);
}

.minimal-checkbox:checked::before {
    content: "";
    width: 8px;
    height: 5px;
    border-left: 1.8px solid #ffffff;
    border-bottom: 1.8px solid #ffffff;
    transform: rotate(-45deg) translate(0.5px, -0.5px);
}

/* Detalhes do Item */
.item-info {
    flex: 1;
    min-width: 0;
}

.item-name {
    font-size: 0.96rem;
    font-weight: 500;
    color: var(--text-primary);
    line-height: 1.35;
    word-break: break-word;
}

.item-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 2px;
    font-size: 0.8rem;
    color: var(--text-secondary);
}

.item-qty-tag {
    font-size: 0.76rem;
    font-weight: 500;
    color: var(--text-secondary);
    background: var(--surface-subtle);
    padding: 1px 6px;
    border-radius: 4px;
}

.item-price {
    font-size: 0.95rem;
    font-weight: 600;
    color: var(--text-primary);
    white-space: nowrap;
    letter-spacing: -0.01em;
}

.item-delete-btn {
    background: none;
    border: none;
    color: #cbd5e1;
    cursor: pointer;
    padding: 6px;
    border-radius: 6px;
    font-size: 0.95rem;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: color 0.15s ease;
}

.item-delete-btn:hover {
    color: #ef4444;
}

/* Cabeçalho de Corredor / Categoria (Discreto e Editorial) */
.aisle-header {
    font-size: 0.72rem;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--text-muted);
    padding: 14px 18px 8px;
    display: flex;
    align-items: center;
    gap: 6px;
    background: #fafbfc;
    border-bottom: 1px solid var(--border-hairline);
}

.aisle-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
}

/* Barra Flutuante Inferior (A exata pílula da imagem anexada!) */
.floating-summary-pill {
    position: fixed;
    bottom: 24px;
    left: 50%;
    transform: translateX(-50%);
    width: calc(100% - 36px);
    max-width: 440px;
    height: 58px;
    background: var(--carbon-dark);
    border-radius: var(--radius-pill);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px 0 22px;
    z-index: 100;
    box-shadow: 0 12px 28px -4px rgba(15, 23, 42, 0.35), 0 6px 12px -4px rgba(15, 23, 42, 0.2);
    border: 1px solid rgba(255, 255, 255, 0.1);
    transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.2s ease;
}

.floating-summary-pill:hover {
    transform: translateX(-50%) translateY(-2px);
    box-shadow: 0 16px 36px -4px rgba(15, 23, 42, 0.4);
}

.floating-summary-text {
    color: #f8fafc;
    font-size: 0.94rem;
    font-weight: 500;
    letter-spacing: -0.01em;
    display: flex;
    align-items: center;
    gap: 6px;
    user-select: none;
}

.floating-summary-text strong {
    color: #ffffff;
    font-weight: 700;
}

.floating-add-btn {
    width: 44px;
    height: 44px;
    border-radius: 50%;
    background: #ffffff;
    color: var(--carbon-dark);
    border: none;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.4rem;
    font-weight: 400;
    cursor: pointer;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.15);
    transition: transform 0.15s ease, background-color 0.15s ease;
    flex-shrink: 0;
}

.floating-add-btn:active {
    transform: scale(0.92);
    background-color: #f1f5f9;
}

/* Bottom Sheet / Gaveta de Adição Rápida */
.drawer-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(15, 23, 42, 0.4);
    backdrop-filter: blur(4px);
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
    border-top-left-radius: 24px;
    border-top-right-radius: 24px;
    padding: 24px 24px 32px;
    box-shadow: 0 -10px 40px rgba(0, 0, 0, 0.15);
    animation: slideUp 0.22s cubic-bezier(0.16, 1, 0.3, 1);
}

.drawer-handle {
    width: 36px;
    height: 4px;
    background: #e2e8f0;
    border-radius: 2px;
    margin: 0 auto 18px;
}

/* Botões do Sistema (Monocromáticos e Elegantes) */
.btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    padding: 10px 18px;
    font-size: 0.9rem;
    font-weight: 600;
    border-radius: var(--radius-pill);
    border: none;
    cursor: pointer;
    transition: all 0.15s ease;
    text-decoration: none;
}

.btn-primary {
    background-color: var(--carbon-dark);
    color: #ffffff;
}

.btn-primary:hover {
    background-color: var(--carbon-hover);
}

.btn-primary:active {
    transform: scale(0.98);
}

.btn-google {
    width: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: #ffffff;
    color: var(--text-primary);
    border: 1px solid var(--border);
    border-radius: var(--radius-btn);
    padding: 10px 16px;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
    font-family: inherit;
}

.btn-google:hover {
    background: var(--surface-hover);
    border-color: #cbd5e1;
    transform: translateY(-1px);
}

.btn-google:active {
    transform: scale(0.98);
}

.btn-secondary {
    background-color: #ffffff;
    color: var(--text-primary);
    border: 1px solid var(--border);
}

.btn-secondary:hover {
    background-color: var(--surface-subtle);
}

.btn-secondary:active {
    transform: scale(0.98);
}

.btn-ghost {
    background: transparent;
    color: var(--text-secondary);
    border: none;
    padding: 6px 10px;
}

.btn-ghost:hover {
    color: var(--text-primary);
    background: var(--surface-subtle);
}

/* Inputs de Formulário */
.form-input {
    width: 100%;
    padding: 12px 14px;
    font-size: 0.95rem;
    border: 1px solid var(--border);
    border-radius: 12px;
    background-color: #ffffff;
    color: var(--text-primary);
    outline: none;
    transition: border-color 0.15s ease, box-shadow 0.15s ease;
    font-family: inherit;
}

.form-input:focus {
    border-color: var(--carbon-dark);
    box-shadow: 0 0 0 1px var(--carbon-dark);
}

.form-label {
    display: block;
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 6px;
}

.form-group {
    margin-bottom: 16px;
}

/* Segmented Control (Alternador estilo iOS/Things) */
.segmented-control {
    display: flex;
    background-color: #f1f5f9;
    padding: 3px;
    border-radius: 10px;
    margin-bottom: 20px;
}

.segmented-btn {
    flex: 1;
    padding: 8px 14px;
    font-size: 0.85rem;
    font-weight: 500;
    color: var(--text-secondary);
    background: transparent;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.15s ease;
    text-align: center;
}

.segmented-btn.active {
    background-color: #ffffff;
    color: var(--text-primary);
    font-weight: 600;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.08);
}

/* Badges Discretos */
.badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 3px 8px;
    border-radius: 6px;
    font-size: 0.75rem;
    font-weight: 500;
    line-height: 1;
}

.badge-code {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-weight: 600;
    background: #f1f5f9;
    color: #334155;
    letter-spacing: 0.05em;
}

/* Alertas de Notificação */
.alert {
    padding: 12px 16px;
    border-radius: 12px;
    font-size: 0.88rem;
    margin-bottom: 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.alert-error {
    background-color: #fef2f2;
    color: #991b1b;
    border: 1px solid #fecaca;
}

.alert-success {
    background-color: #f0fdf4;
    color: #166534;
    border: 1px solid #bbf7d0;
}

/* Modais Clássicos */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(15, 23, 42, 0.45);
    backdrop-filter: blur(6px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 150;
    padding: 20px;
    animation: fadeIn 0.15s ease-out;
}

.modal-content {
    background-color: #ffffff;
    border-radius: 20px;
    width: 100%;
    max-width: 480px;
    padding: 24px;
    box-shadow: 0 20px 40px -10px rgba(0, 0, 0, 0.2);
    animation: scaleIn 0.2s cubic-bezier(0.16, 1, 0.3, 1);
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
"#;
