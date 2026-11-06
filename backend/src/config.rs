use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: u16,
    pub database_url: Option<String>,
    pub jwt_secret: String,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    pub google_client_id: Option<String>,
    pub google_client_secret: Option<String>,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);

        let database_url = std::env::var("DATABASE_URL").ok().filter(|s| !s.is_empty());
        let jwt_secret = std::env::var("SUPABASE_JWT_SECRET")
            .or_else(|_| std::env::var("JWT_SECRET"))
            .unwrap_or_else(|_| "default-development-jwt-secret-min-32-chars-long!".to_string());

        let supabase_url = std::env::var("SUPABASE_URL").ok();
        let supabase_anon_key = std::env::var("SUPABASE_ANON_KEY").ok();

        let google_client_id = std::env::var("GOOGLE_CLIENT_ID").ok();
        let google_client_secret = std::env::var("GOOGLE_CLIENT_SECRET").ok();

        Self {
            port,
            database_url,
            jwt_secret,
            supabase_url,
            supabase_anon_key,
            google_client_id,
            google_client_secret,
        }
    }
}

pub type SharedConfig = Arc<Config>;
