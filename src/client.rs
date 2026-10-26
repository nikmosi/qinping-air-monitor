use std::sync::Arc;
use std::time::{Duration, Instant};
use base64::Engine;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, error, info};

const DEFAULT_AUTH_URL: &str = "https://oauth.cleargrass.com/oauth2/token";
const DEFAULT_API_URL: &str = "https://apis.cleargrass.com/v1/apis/devices";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QingpingDevicesResponse {
    #[serde(default)]
    pub total: usize,
    #[serde(default)]
    pub devices: Vec<QingpingDevice>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct QingpingDevice {
    pub info: DeviceInfo,
    #[serde(default)]
    pub data: Option<SensorData>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceInfo {
    pub mac: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub status: Option<DeviceStatus>,
    #[serde(default)]
    pub product: Option<ProductInfo>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DeviceStatus {
    #[serde(default)]
    pub offline: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ProductInfo {
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub en_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SensorData {
    #[serde(default)]
    pub timestamp: Option<TimestampValue>,
    #[serde(default)]
    pub battery: Option<NumericValue>,
    #[serde(default)]
    pub temperature: Option<NumericValue>,
    #[serde(default)]
    pub humidity: Option<NumericValue>,
    #[serde(default)]
    pub co2: Option<NumericValue>,
    #[serde(default)]
    pub pm25: Option<NumericValue>,
    #[serde(default)]
    pub pm10: Option<NumericValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TimestampValue {
    pub value: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct NumericValue {
    pub value: f64,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
}

struct CachedToken {
    token: String,
    expires_at: Instant,
}

struct CachedData {
    data: QingpingDevicesResponse,
    fetched_at: Instant,
}

pub struct QingpingClient {
    http: reqwest::Client,
    app_key: String,
    app_secret: String,
    auth_url: String,
    api_url: String,
    cache_ttl: Duration,
    cached_token: Arc<RwLock<Option<CachedToken>>>,
    cached_devices: Arc<RwLock<Option<CachedData>>>,
}

impl QingpingClient {
    pub fn new(app_key: impl Into<String>, app_secret: impl Into<String>, cache_ttl_secs: u64) -> Self {
        Self::with_endpoints(app_key, app_secret, cache_ttl_secs, DEFAULT_AUTH_URL, DEFAULT_API_URL)
    }

    pub fn with_endpoints(
        app_key: impl Into<String>,
        app_secret: impl Into<String>,
        cache_ttl_secs: u64,
        auth_url: impl Into<String>,
        api_url: impl Into<String>,
    ) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        Self {
            http,
            app_key: app_key.into(),
            app_secret: app_secret.into(),
            auth_url: auth_url.into(),
            api_url: api_url.into(),
            cache_ttl: Duration::from_secs(cache_ttl_secs),
            cached_token: Arc::new(RwLock::new(None)),
            cached_devices: Arc::new(RwLock::new(None)),
        }
    }

    async fn get_access_token(&self) -> Result<String, String> {
        // Check cached token
        {
            let cache = self.cached_token.read().await;
            if let Some(ref t) = *cache {
                // Buffer 60 seconds before actual expiration
                if t.expires_at > Instant::now() + Duration::from_secs(60) {
                    return Ok(t.token.clone());
                }
            }
        }

        let mut cache_write = self.cached_token.write().await;
        // Double check cache after acquiring write lock
        if let Some(ref t) = *cache_write {
            if t.expires_at > Instant::now() + Duration::from_secs(60) {
                return Ok(t.token.clone());
            }
        }

        debug!("Fetching new Qingping OAuth2 access token");
        let basic_auth = base64::engine::general_purpose::STANDARD
            .encode(format!("{}:{}", self.app_key, self.app_secret));

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Basic {}", basic_auth))
                .map_err(|e| format!("Invalid auth header: {}", e))?,
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let params = [
            ("grant_type", "client_credentials"),
            ("scope", "device_full_access"),
        ];

        let resp = self
            .http
            .post(&self.auth_url)
            .headers(headers)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(format!("Token request returned HTTP {}: {}", status, body));
        }

        let token_data: TokenResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse token response: {}", e))?;

        let expires_at = Instant::now() + Duration::from_secs(token_data.expires_in);
        let token = token_data.access_token.clone();

        *cache_write = Some(CachedToken {
            token: token_data.access_token,
            expires_at,
        });

        info!("Successfully obtained new Qingping OAuth2 access token");
        Ok(token)
    }

    pub async fn fetch_devices(&self) -> Result<QingpingDevicesResponse, String> {
        // 1. Check TTL cache
        {
            let cache = self.cached_devices.read().await;
            if let Some(ref cd) = *cache {
                if cd.fetched_at.elapsed() < self.cache_ttl {
                    debug!("Returning cached Qingping device telemetry");
                    return Ok(cd.data.clone());
                }
            }
        }

        let token = self.get_access_token().await?;
        debug!("Fetching device list from Qingping API");

        let resp = self
            .http
            .get(&self.api_url)
            .header(AUTHORIZATION, format!("Bearer {}", token))
            .send()
            .await
            .map_err(|e| format!("Device request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            error!("Failed to fetch devices: HTTP {}: {}", status, body);
            return Err(format!("Devices request returned HTTP {}: {}", status, body));
        }

        let data: QingpingDevicesResponse = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse devices response: {}", e))?;

        // Update TTL cache
        {
            let mut cache = self.cached_devices.write().await;
            *cache = Some(CachedData {
                data: data.clone(),
                fetched_at: Instant::now(),
            });
        }

        Ok(data)
    }
}
