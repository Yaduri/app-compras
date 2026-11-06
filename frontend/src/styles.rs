pub const APP_CSS: &str = r#"
/* Reset e Base Global */
*, *::before, *::after {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    -webkit-tap-highlight-color: transparent;
}

html, body {
    width: 100%;
    height: 100%;
    background-color: #f8fafc;
    color: #0f172a;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", "Inter", Roboto, Helvetica, Arial, sans-serif;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    line-height: 1.5;
}

/* Shell do App: Edge-to-Edge sem faixas cinzas estranhas */
.app-container {
    width: 100%;
    min-height: 100vh;
    background-color: #f8fafc;
    display: flex;
    flex-direction: column;
    margin: 0 auto;
    position: relative;
}

/* Header & Barra Superior com Efeito Glassmorphism */
.navbar {
    background: rgba(255, 255, 255, 0.92);
    backdrop-filter: blur(16px);
    -webkit-backdrop-filter: blur(16px);
    border-bottom: 1px solid rgba(226, 232, 240, 0.85);
    padding: 14px 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: sticky;
    top: 0;
    z-index: 50;
    box-shadow: 0 1px 3px 0 rgba(0, 0, 0, 0.03);
}

.navbar h1 {
    font-size: 1.15rem;
    font-weight: 800;
    letter-spacing: -0.025em;
    color: #0f172a;
    display: flex;
    align-items: center;
    gap: 10px;
}

.brand-badge {
    background: linear-gradient(135deg, #10b981 0%, #059669 100%);
    color: #ffffff;
    width: 32px;
    height: 32px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1rem;
    box-shadow: 0 2px 8px rgba(16, 185, 129, 0.35);
}

.live-indicator {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 0.72rem;
    font-weight: 700;
    color: #059669;
    background: #ecfdf5;
    padding: 2px 8px;
    border-radius: 9999px;
    border: 1px solid #a7f3d0;
}

.live-dot {
    width: 6px;
    height: 6px;
    background-color: #10b981;
    border-radius: 50%;
    box-shadow: 0 0 0 2px rgba(16, 185, 129, 0.2);
}

.nav-actions {
    display: flex;
    align-items: center;
    gap: 10px;
}

.user-chip {
    display: flex;
    align-items: center;
    gap: 8px;
    background: #f1f5f9;
    padding: 5px 12px 5px 6px;
    border-radius: 9999px;
    border: 1px solid #e2e8f0;
    font-size: 0.85rem;
    font-weight: 600;
    color: #334155;
}

.user-avatar-circle {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: linear-gradient(135deg, #059669 0%, #10b981 100%);
    color: #ffffff;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.75rem;
    font-weight: 800;
}

.btn-icon {
    background: #f1f5f9;
    border: 1px solid #e2e8f0;
    color: #475569;
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    padding: 6px 12px;
    border-radius: 8px;
    transition: all 0.15s ease;
    display: inline-flex;
    align-items: center;
    gap: 6px;
}

.btn-icon:hover {
    background: #e2e8f0;
    color: #0f172a;
}

/* Área Principal de Conteúdo */
.main-content {
    flex: 1;
    padding: 20px;
    max-width: 620px;
    width: 100%;
    margin: 0 auto;
}

/* Cards Elegantes com Borda Delicada e Sombras Suaves */
.card {
    background: #ffffff;
    border: 1px solid rgba(226, 232, 240, 0.85);
    border-radius: 18px;
    padding: 20px;
    margin-bottom: 16px;
    box-shadow: 0 4px 20px -2px rgba(15, 23, 42, 0.04), 0 1px 3px rgba(0, 0, 0, 0.02);
    transition: transform 0.2s cubic-bezier(0.16, 1, 0.3, 1), box-shadow 0.2s cubic-bezier(0.16, 1, 0.3, 1);
}

.card:hover {
    box-shadow: 0 8px 24px -4px rgba(15, 23, 42, 0.08);
}

/* Botões Modernos e Táteis */
.btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-weight: 700;
    font-size: 0.95rem;
    padding: 12px 20px;
    border-radius: 12px;
    border: none;
    cursor: pointer;
    transition: all 0.18s cubic-bezier(0.16, 1, 0.3, 1);
    user-select: none;
    min-height: 48px;
}

.btn:active {
    transform: scale(0.97);
}

.btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
    transform: none;
}

.btn-primary {
    background: linear-gradient(135deg, #10b981 0%, #059669 100%);
    color: #ffffff;
    box-shadow: 0 4px 14px rgba(16, 185, 129, 0.35);
}

.btn-primary:hover:not(:disabled) {
    background: linear-gradient(135deg, #059669 0%, #047857 100%);
    box-shadow: 0 6px 18px rgba(16, 185, 129, 0.42);
    transform: translateY(-1px);
}

.btn-secondary {
    background-color: #f1f5f9;
    color: #334155;
    border: 1px solid #cbd5e1;
}

.btn-secondary:hover:not(:disabled) {
    background-color: #e2e8f0;
    color: #0f172a;
}

.btn-success {
    background: linear-gradient(135deg, #10b981 0%, #059669 100%);
    color: #ffffff;
}

.btn-danger {
    background: linear-gradient(135deg, #f43f5e 0%, #e11d48 100%);
    color: #ffffff;
}

.btn-danger:hover:not(:disabled) {
    background: #be123c;
}

/* Segmented Control Pills (iOS Style) */
.segmented-control {
    background: #f1f5f9;
    padding: 4px;
    border-radius: 14px;
    display: flex;
    gap: 4px;
    margin-bottom: 20px;
}

.segmented-btn {
    flex: 1;
    padding: 10px 14px;
    border-radius: 10px;
    font-size: 0.9rem;
    font-weight: 600;
    border: none;
    cursor: pointer;
    background: transparent;
    color: #64748b;
    transition: all 0.2s ease;
}

.segmented-btn.active {
    background: #ffffff;
    color: #0f172a;
    font-weight: 700;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

/* Formulários e Inputs */
.form-group {
    margin-bottom: 18px;
}

.form-label {
    display: block;
    font-size: 0.85rem;
    font-weight: 700;
    color: #334155;
    margin-bottom: 6px;
    letter-spacing: -0.01em;
}

.form-input {
    width: 100%;
    padding: 13px 16px;
    font-size: 0.95rem;
    border: 1.5px solid #e2e8f0;
    border-radius: 12px;
    background-color: #ffffff;
    color: #0f172a;
    outline: none;
    transition: border-color 0.2s, box-shadow 0.2s;
}

.form-input:focus {
    border-color: #10b981;
    box-shadow: 0 0 0 4px rgba(16, 185, 129, 0.12);
}

/* Badges e Tags */
.badge {
    display: inline-flex;
    align-items: center;
    gap: 5px;
    font-size: 0.75rem;
    font-weight: 700;
    padding: 3px 10px;
    border-radius: 9999px;
    text-transform: uppercase;
    letter-spacing: 0.03em;
}

.badge-code {
    background: #f0fdf4;
    color: #059669;
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
    font-size: 0.85rem;
    font-weight: 700;
    padding: 4px 10px;
    border: 1.5px dashed #10b981;
    border-radius: 8px;
    cursor: pointer;
    transition: background 0.15s;
}

.badge-code:hover {
    background: #dcfce7;
}

/* Item Card (Lista de Compras) */
.item-card {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 15px 18px;
    background: #ffffff;
    border: 1.5px solid #f1f5f9;
    border-radius: 14px;
    margin-bottom: 10px;
    box-shadow: 0 2px 6px -1px rgba(0, 0, 0, 0.02);
    transition: all 0.2s ease;
}

.item-card:hover {
    border-color: #e2e8f0;
    box-shadow: 0 4px 12px -2px rgba(0, 0, 0, 0.05);
}

.item-card.purchased {
    background-color: #f8fafc;
    border-color: #f1f5f9;
    opacity: 0.65;
}

.item-card.purchased .item-name {
    text-decoration: line-through;
    color: #94a3b8;
}

.item-checkbox {
    width: 26px;
    height: 26px;
    border-radius: 8px;
    cursor: pointer;
    accent-color: #10b981;
    flex-shrink: 0;
}

.item-details {
    flex: 1;
}

.item-name {
    font-size: 1.02rem;
    font-weight: 700;
    color: #0f172a;
    letter-spacing: -0.015em;
}

.item-meta {
    font-size: 0.82rem;
    color: #64748b;
    margin-top: 4px;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
}

.item-price-tag {
    font-weight: 700;
    color: #059669;
    background: #ecfdf5;
    padding: 1px 7px;
    border-radius: 6px;
}

/* Categorias Agrupadas */
.category-group-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 4px 6px 4px;
    margin-top: 18px;
    margin-bottom: 8px;
}

.category-group-header h3 {
    font-size: 0.92rem;
    font-weight: 800;
    color: #475569;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 8px;
}

/* Barra de Progresso Elegante */
.progress-container {
    width: 100%;
    height: 8px;
    background-color: #e2e8f0;
    border-radius: 9999px;
    overflow: hidden;
    margin: 10px 0 6px 0;
}

.progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #10b981 0%, #059669 100%);
    border-radius: 9999px;
    transition: width 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

/* Modais e Overlays */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(15, 23, 42, 0.55);
    backdrop-filter: blur(6px);
    -webkit-backdrop-filter: blur(6px);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    z-index: 100;
}

.modal-content {
    background: #ffffff;
    border-radius: 22px;
    padding: 26px;
    max-width: 480px;
    width: 100%;
    box-shadow: 0 20px 40px -10px rgba(0, 0, 0, 0.25);
    animation: modalPop 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

@keyframes modalPop {
    from {
        opacity: 0;
        transform: scale(0.95) translateY(10px);
    }
    to {
        opacity: 1;
        transform: scale(1) translateY(0);
    }
}

/* Alertas e Notificações */
.alert {
    padding: 13px 18px;
    border-radius: 12px;
    margin-bottom: 16px;
    font-size: 0.9rem;
    font-weight: 600;
    display: flex;
    align-items: center;
    gap: 10px;
}

.alert-error {
    background-color: #fef2f2;
    color: #dc2626;
    border: 1px solid #fecaca;
}

.alert-success {
    background-color: #f0fdf4;
    color: #15803d;
    border: 1px solid #bbf7d0;
}

/* Modo Supermercado (Alto Contraste com 1 Mão) */
.market-mode {
    background-color: #0b1320 !important;
    color: #f8fafc !important;
}

.market-mode .navbar {
    background: rgba(15, 23, 42, 0.95);
    border-bottom: 1px solid #1e293b;
}

.market-mode .navbar h1 {
    color: #f8fafc;
}

.market-mode .card {
    background-color: #151f30;
    border-color: #1e293b;
    color: #f8fafc;
}

.market-mode .item-card {
    background-color: #151f30;
    border-color: #1e293b;
    padding: 18px 18px;
    margin-bottom: 12px;
}

.market-mode .item-card.purchased {
    background-color: #0b1320;
    border-color: #151f30;
    opacity: 0.5;
}

.market-mode .item-name {
    font-size: 1.25rem;
    font-weight: 800;
    color: #ffffff;
}

.market-mode .item-checkbox {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    accent-color: #10b981;
}

.market-mode .item-meta {
    font-size: 0.95rem;
    color: #94a3b8;
}

.market-mode h2, .market-mode h3, .market-mode h4 {
    color: #f8fafc !important;
}

/* Sticky Bottom Summary no Modo Supermercado */
.sticky-cart-summary {
    position: sticky;
    bottom: 0;
    left: 0;
    right: 0;
    background: rgba(15, 23, 42, 0.95);
    backdrop-filter: blur(12px);
    border-top: 1px solid #1e293b;
    color: #ffffff;
    padding: 16px 22px;
    box-shadow: 0 -8px 24px rgba(0, 0, 0, 0.35);
    display: flex;
    justify-content: space-between;
    align-items: center;
    z-index: 40;
}
"#;
