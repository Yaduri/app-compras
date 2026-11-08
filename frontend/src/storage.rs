use serde::{Deserialize, Serialize};
use shared::dto::UserProfile;

const SESSION_FILE: &str = "auth_session.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub token: String,
    pub user: UserProfile,
}

/// Pasta privada do app para guardar a sessão
#[cfg(not(target_arch = "wasm32"))]
fn session_dir() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        // Em aparelhos com vários usuários (perfil de trabalho, etc.) a pasta do app fica em
        // /data/user/<id>/<pacote>. O id do usuário Android = UID do processo / 100000.
        use std::os::unix::fs::MetadataExt;
        const PACKAGE: &str = "com.appcompras.lista";
        let android_user = std::fs::metadata("/proc/self")
            .map(|m| m.uid() / 100_000)
            .unwrap_or(0);
        let per_user = std::path::PathBuf::from(format!("/data/user/{}/{}/files", android_user, PACKAGE));
        if per_user.parent().map(|p| p.exists()).unwrap_or(false) {
            return per_user;
        }
        std::path::PathBuf::from(format!("/data/data/{}/files", PACKAGE))
    }

    #[cfg(not(target_os = "android"))]
    {
        // Desktop: pasta de configuração do usuário (a pasta temporária é compartilhada)
        let base = std::env::var_os("APPDATA")
            .map(std::path::PathBuf::from)
            .or_else(|| std::env::var_os("XDG_CONFIG_HOME").map(std::path::PathBuf::from))
            .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))
            .unwrap_or_else(std::env::temp_dir);
        base.join("app-compras")
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn session_path() -> std::path::PathBuf {
    session_dir().join(SESSION_FILE)
}

/// Carrega a sessão do sistema de arquivos nativo (instantâneo na inicialização)
pub fn load_session_fs() -> Option<StoredSession> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let content = std::fs::read_to_string(session_path()).ok()?;
        let sess = serde_json::from_str::<StoredSession>(&content).ok()?;
        if !sess.token.trim().is_empty() {
            return Some(sess);
        }
    }
    None
}

/// Salva a sessão no localStorage e no sistema de arquivos local
pub fn save_session(token: &str, user: &UserProfile) {
    let sess = StoredSession {
        token: token.to_string(),
        user: user.clone(),
    };

    let Ok(json_str) = serde_json::to_string(&sess) else {
        return;
    };

    // 1. localStorage (necessário na versão web). O valor vai serializado como string JSON.
    if let Ok(json_quoted) = serde_json::to_string(&json_str) {
        let js = format!(
            r#"try {{ localStorage.setItem("app_compras_session", {}); }} catch(e) {{ console.error("storage error", e); }}"#,
            json_quoted
        );
        let _ = dioxus::prelude::document::eval(&js);
    }

    // 2. Arquivo na pasta privada do app, legível só pelo próprio usuário
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = session_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if std::fs::write(&path, json_str).is_ok() {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
            }
        }
    }
}

/// Remove a sessão do localStorage e do sistema de arquivos nativo
pub fn clear_session() {
    let _ = dioxus::prelude::document::eval(
        r#"
        try {
            localStorage.removeItem("app_compras_session");
        } catch(e) {}
        "#,
    );

    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = std::fs::remove_file(session_path());
    }
}

// --- Login com Google pendente ---
//
// O token do Google chega por deep link (appcompras://auth#access_token=...), que qualquer
// site ou app consegue abrir. Para impedir que um terceiro troque a conta logada, o app só
// aceita esse token se o próprio usuário tiver tocado em "Entrar com Google" há pouco tempo.

const OAUTH_PENDING_FILE: &str = "oauth_pending";
const OAUTH_PENDING_KEY: &str = "app_compras_oauth_pending";
/// Tempo máximo entre tocar em "Entrar com Google" e o retorno ao app
const OAUTH_PENDING_MAX_SECS: i64 = 10 * 60;

/// Registra que o usuário iniciou o login com Google agora
pub fn mark_oauth_pending() {
    let now = chrono::Utc::now().timestamp();

    let _ = dioxus::prelude::document::eval(&format!(
        r#"try {{ localStorage.setItem("{OAUTH_PENDING_KEY}", "{now}"); }} catch(e) {{}}"#
    ));

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = session_dir().join(OAUTH_PENDING_FILE);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(path, now.to_string());
    }
}

/// Consome a marcação (uso único). `true` se havia um login iniciado dentro do prazo.
pub async fn take_oauth_pending() -> bool {
    #[cfg(not(target_arch = "wasm32"))]
    let mut started: Option<i64> = {
        let path = session_dir().join(OAUTH_PENDING_FILE);
        let value = std::fs::read_to_string(&path).ok().and_then(|s| s.trim().parse().ok());
        let _ = std::fs::remove_file(path);
        value
    };
    #[cfg(target_arch = "wasm32")]
    let mut started: Option<i64> = None;

    let mut eval = dioxus::prelude::document::eval(&format!(
        r#"
        let v = "";
        try {{ v = localStorage.getItem("{OAUTH_PENDING_KEY}") || ""; localStorage.removeItem("{OAUTH_PENDING_KEY}"); }} catch(e) {{}}
        dioxus.send(v);
        "#
    ));
    if started.is_none() {
        if let Ok(v) = eval.recv::<String>().await {
            started = v.trim().parse().ok();
        }
    }

    let now = chrono::Utc::now().timestamp();
    started.is_some_and(|t| (0..=OAUTH_PENDING_MAX_SECS).contains(&(now - t)))
}
