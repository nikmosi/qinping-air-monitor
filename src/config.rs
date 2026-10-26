use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub host: String,
    pub port: u16,
    pub app_key: String,
    pub app_secret: String,
    pub cache_ttl_secs: u64,
    pub log_level: String,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8000);
        let app_key = env::var("QINGPING_APP_KEY").unwrap_or_default();
        let app_secret = env::var("QINGPING_APP_SECRET").unwrap_or_default();
        let cache_ttl_secs = env::var("METRICS_CACHE_TTL_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);
        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());

        Self {
            host,
            port,
            app_key,
            app_secret,
            cache_ttl_secs,
            log_level,
        }
    }
}
