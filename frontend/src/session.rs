//! Estado global compartilhado entre as telas via contexto do Dioxus

use dioxus::prelude::*;

use crate::api::ApiError;

/// Sinaliza que o servidor recusou o token (401): o App faz logout e volta ao login
#[derive(Clone, Copy)]
pub struct SessionExpired(pub Signal<bool>);

/// Indica se a sincronização em tempo real (WebSocket) está conectada
#[derive(Clone, Copy)]
pub struct LiveStatus(pub Signal<bool>);

/// Exibe o erro de uma chamada à API. Em caso de sessão expirada, dispara o logout.
pub fn show_api_error(err: ApiError, mut error_msg: Signal<Option<String>>, session: SessionExpired) {
    if err.is_unauthorized() {
        let mut expired = session.0;
        expired.set(true);
    } else {
        error_msg.set(Some(err.message));
    }
}

/// Espera sem travar a interface
pub async fn sleep_ms(ms: u64) {
    #[cfg(not(target_arch = "wasm32"))]
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;

    #[cfg(target_arch = "wasm32")]
    {
        let mut eval = document::eval(&format!("setTimeout(() => dioxus.send(true), {ms});"));
        let _ = eval.recv::<bool>().await;
    }
}

// --- Botão/gesto "voltar" do Android ---
//
// O Android (WryActivity) chama `webView.goBack()` quando há histórico. Enquanto houver
// alguma camada aberta (lista, gaveta, modal), mantemos UMA entrada extra no histórico;
// o "voltar" dispara `popstate`, e fechamos só a camada do topo.

/// Camadas que o "voltar" fecha, da mais externa para a mais interna
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    List,
    AddDrawer,
    Suggestions,
    DeleteList,
    CreateList,
    JoinList,
}

#[derive(Clone, Copy)]
pub struct BackStack {
    stack: Signal<Vec<Layer>>,
    /// Última camada fechada pelo "voltar" (com contador, para repetir a mesma camada)
    pub closed: Signal<Option<(Layer, u64)>>,
    ignore_pops: Signal<u32>,
}

impl BackStack {
    pub fn new() -> Self {
        Self {
            stack: Signal::new(Vec::new()),
            closed: Signal::new(None),
            ignore_pops: Signal::new(0),
        }
    }

    fn push_history_entry() {
        let _ = document::eval(r#"try { history.pushState({ appCompras: true }, ""); } catch (e) {}"#);
    }

    /// Registra uma camada aberta
    pub fn open(mut self, layer: Layer) {
        let was_empty = self.stack.peek().is_empty();
        if self.stack.peek().contains(&layer) {
            return;
        }
        self.stack.write().push(layer);
        if was_empty {
            Self::push_history_entry();
        }
    }

    /// Camada fechada pela própria interface (botão, toque fora): fecha também as de cima
    pub fn close(mut self, layer: Layer) {
        let Some(pos) = self.stack.peek().iter().position(|l| *l == layer) else {
            return;
        };
        self.stack.write().truncate(pos);
        if self.stack.peek().is_empty() {
            // Remove a entrada extra do histórico sem tratar como "voltar"
            *self.ignore_pops.write() += 1;
            let _ = document::eval("try { history.back(); } catch (e) {}");
        }
    }

    /// Chamado a cada `popstate` (botão/gesto voltar)
    pub fn on_back(mut self) {
        if *self.ignore_pops.peek() > 0 {
            *self.ignore_pops.write() -= 1;
            return;
        }
        let top = self.stack.write().pop();
        if let Some(layer) = top {
            let seq = self.closed.peek().as_ref().map(|(_, s)| s + 1).unwrap_or(1);
            self.closed.set(Some((layer, seq)));
            if !self.stack.peek().is_empty() {
                Self::push_history_entry();
            }
        }
    }

    /// Esquece todas as camadas (ex.: logout)
    pub fn reset(mut self) {
        let had_layers = !self.stack.peek().is_empty();
        self.stack.write().clear();
        if had_layers {
            *self.ignore_pops.write() += 1;
            let _ = document::eval("try { history.back(); } catch (e) {}");
        }
    }
}

impl Default for BackStack {
    fn default() -> Self {
        Self::new()
    }
}

/// Liga um `Signal<bool>` de visibilidade a uma camada do "voltar":
/// abrir registra a camada; o "voltar" do sistema fecha (põe `false`).
pub fn use_back_layer(mut visible: Signal<bool>, layer: Layer) {
    let back = use_context::<BackStack>();

    use_effect(move || {
        if *visible.read() {
            back.open(layer);
        } else {
            back.close(layer);
        }
    });

    use_effect(move || {
        if let Some((closed, _)) = *back.closed.read() {
            if closed == layer && *visible.peek() {
                visible.set(false);
            }
        }
    });

    use_drop(move || back.close(layer));
}
