use serde::{Deserialize, Serialize};
use shared::dto::UserProfile;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredSession {
    pub token: String,
    pub user: UserProfile,
}

#[cfg(not(target_arch = "wasm32"))]
fn get_file_path() -> std::path::PathBuf {
    #[cfg(target_os = "android")]
    {
        std::path::PathBuf::from("/data/data/com.appcompras.lista/files/auth_session.json")
    }
    #[cfg(not(target_os = "android"))]
    {
        std::env::temp_dir().join("app_compras_auth_session.json")
    }
}

/// Carrega a sessão do sistema de arquivos nativo (instantâneo na inicialização)
pub fn load_session_fs() -> Option<StoredSession> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = get_file_path();
        if let Ok(content) = std::fs::read_to_string(path) {
            if let Ok(sess) = serde_json::from_str::<StoredSession>(&content) {
                if !sess.token.trim().is_empty() {
                    return Some(sess);
                }
            }
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

    if let Ok(json_str) = serde_json::to_string(&sess) {
        // 1. Salva no localStorage via document::eval
        if let Ok(json_quoted) = serde_json::to_string(&json_str) {
            let js = format!(
                r#"try {{ localStorage.setItem("app_compras_session", {}); }} catch(e) {{ console.error("storage error", e); }}"#,
                json_quoted
            );
            let _ = dioxus::prelude::document::eval(&js);
        }

        // 2. Salva no sistema de arquivos nativo
        #[cfg(not(target_arch = "wasm32"))]
        {
            let path = get_file_path();
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(path, json_str);
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
        let path = get_file_path();
        let _ = std::fs::remove_file(path);
    }
}
