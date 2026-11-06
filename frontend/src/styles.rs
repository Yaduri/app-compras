pub const APP_CSS: &str = r#"
* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
    -webkit-tap-highlight-color: transparent;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
    background-color: #f1f5f9;
    color: #0f172a;
    line-height: 1.5;
}

.app-container {
    max-width: 680px;
    margin: 0 auto;
    min-height: 100vh;
    background-color: #ffffff;
    box-shadow: 0 4px 20px -2px rgba(0, 0, 0, 0.08);
    display: flex;
    flex-direction: column;
}

/* Header & Navbar */
.navbar {
    background-color: #0284c7;
    color: #ffffff;
    padding: 16px 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    position: sticky;
    top: 0;
    z-index: 50;
    box-shadow: 0 2px 8px rgba(0,0,0,0.12);
}

.navbar h1 {
    font-size: 1.25rem;
    font-weight: 700;
    display: flex;
    align-items: center;
    gap: 8px;
}

.nav-actions {
    display: flex;
    align-items: center;
    gap: 12px;
}

.btn-icon {
    background: transparent;
    border: none;
    color: #ffffff;
    cursor: pointer;
    font-size: 1.1rem;
    padding: 6px 10px;
    border-radius: 8px;
    transition: background-color 0.2s;
}

.btn-icon:hover {
    background: rgba(255, 255, 255, 0.2);
}

/* Content Area */
.main-content {
    flex: 1;
    padding: 20px;
}

/* Cards */
.card {
    background: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 14px;
    padding: 16px;
    margin-bottom: 16px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.04);
    transition: transform 0.15s, box-shadow 0.15s;
}

.card:hover {
    box-shadow: 0 4px 12px rgba(0,0,0,0.07);
}

/* Buttons */
.btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    font-weight: 600;
    font-size: 0.95rem;
    padding: 12px 18px;
    border-radius: 10px;
    border: none;
    cursor: pointer;
    transition: background-color 0.15s, transform 0.1s;
    user-select: none;
    min-height: 48px;
}

.btn:active {
    transform: scale(0.98);
}

.btn-primary {
    background-color: #0284c7;
    color: #ffffff;
}

.btn-primary:hover {
    background-color: #0369a1;
}

.btn-secondary {
    background-color: #f1f5f9;
    color: #334155;
    border: 1px solid #cbd5e1;
}

.btn-secondary:hover {
    background-color: #e2e8f0;
}

.btn-success {
    background-color: #16a34a;
    color: #ffffff;
}

.btn-success:hover {
    background-color: #15803d;
}

.btn-danger {
    background-color: #ef4444;
    color: #ffffff;
}

.btn-danger:hover {
    background-color: #dc2626;
}

/* Badges & Chips */
.badge {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 3px 8px;
    border-radius: 9999px;
    text-transform: uppercase;
}

.badge-code {
    background-color: #e0f2fe;
    color: #0369a1;
    font-family: monospace;
    font-size: 0.85rem;
    padding: 4px 8px;
    border: 1px dashed #38bdf8;
    cursor: pointer;
}

/* Forms & Inputs */
.form-group {
    margin-bottom: 16px;
}

.form-label {
    display: block;
    font-size: 0.875rem;
    font-weight: 600;
    color: #475569;
    margin-bottom: 6px;
}

.form-input {
    width: 100%;
    padding: 12px 14px;
    font-size: 1rem;
    border: 1px solid #cbd5e1;
    border-radius: 10px;
    outline: none;
    transition: border-color 0.2s, box-shadow 0.2s;
}

.form-input:focus {
    border-color: #0284c7;
    box-shadow: 0 0 0 3px rgba(2, 132, 199, 0.15);
}

/* Checkbox Grande para Supermercado (Uso com 1 mão) */
.market-checkbox {
    width: 28px;
    height: 28px;
    border-radius: 8px;
    cursor: pointer;
    accent-color: #16a34a;
    flex-shrink: 0;
}

/* Item Card */
.item-card {
    display: flex;
    align-items: center;
    gap: 14px;
    padding: 14px;
    background: #ffffff;
    border: 1px solid #e2e8f0;
    border-radius: 12px;
    margin-bottom: 10px;
    transition: background-color 0.15s, opacity 0.15s;
}

.item-card.purchased {
    background-color: #f8fafc;
    border-color: #f1f5f9;
    opacity: 0.7;
}

.item-card.purchased .item-name {
    text-decoration: line-through;
    color: #64748b;
}

.item-details {
    flex: 1;
}

.item-name {
    font-size: 1.05rem;
    font-weight: 600;
    color: #0f172a;
}

.item-meta {
    font-size: 0.8rem;
    color: #64748b;
    margin-top: 4px;
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
}

/* Progress Bar */
.progress-container {
    width: 100%;
    height: 8px;
    background-color: #e2e8f0;
    border-radius: 9999px;
    overflow: hidden;
    margin: 8px 0;
}

.progress-bar {
    height: 100%;
    background-color: #16a34a;
    transition: width 0.3s ease;
}

/* Modal Overlay */
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 20px;
    z-index: 100;
}

.modal-content {
    background: #ffffff;
    border-radius: 16px;
    padding: 24px;
    max-width: 480px;
    width: 100%;
    box-shadow: 0 10px 25px rgba(0, 0, 0, 0.2);
}

/* Notifications */
.alert {
    padding: 12px 16px;
    border-radius: 10px;
    margin-bottom: 16px;
    font-size: 0.9rem;
    font-weight: 500;
}

.alert-error {
    background-color: #fef2f2;
    color: #b91c1c;
    border: 1px solid #fecaca;
}

.alert-success {
    background-color: #f0fdf4;
    color: #15803d;
    border: 1px solid #bbf7d0;
}

/* Modo Supermercado (Alto Contraste e Fontes Maiores para Uso com 1 Mão) */
.market-mode {
    background-color: #0f172a !important;
    color: #f8fafc !important;
}

.market-mode .navbar {
    background-color: #0369a1;
}

.market-mode .card {
    background-color: #1e293b;
    border-color: #334155;
    color: #f8fafc;
}

.market-mode .item-card {
    background-color: #1e293b;
    border-color: #334155;
    padding: 18px 16px;
    margin-bottom: 14px;
}

.market-mode .item-card.purchased {
    background-color: #0f172a;
    border-color: #1e293b;
    opacity: 0.55;
}

.market-mode .item-name {
    font-size: 1.25rem;
    font-weight: 700;
    color: #ffffff;
}

.market-mode .market-checkbox {
    width: 36px;
    height: 36px;
    border-radius: 10px;
}

.market-mode .item-meta {
    font-size: 0.95rem;
    color: #94a3b8;
}

.market-mode h2, .market-mode h3, .market-mode h4 {
    color: #f8fafc !important;
}

.sticky-cart-summary {
    position: sticky;
    bottom: 0;
    left: 0;
    right: 0;
    background: #0284c7;
    color: #ffffff;
    padding: 14px 20px;
    box-shadow: 0 -4px 16px rgba(0,0,0,0.15);
    display: flex;
    justify-content: space-between;
    align-items: center;
    z-index: 40;
}

.market-mode .sticky-cart-summary {
    background: #047857;
}

"#;
