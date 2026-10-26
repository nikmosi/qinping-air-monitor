use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use qinping_air_monitor::{
    client::{DeviceInfo, DeviceStatus, NumericValue, ProductInfo, QingpingDevice, QingpingDevicesResponse, SensorData, TimestampValue},
    create_app,
    metrics::render_prometheus_metrics,
    AppConfig, AppState,
};
use tower::ServiceExt;

#[tokio::test]
async fn test_healthz_endpoint() {
    let config = AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8000,
        app_key: "dummy".to_string(),
        app_secret: "dummy".to_string(),
        cache_ttl_secs: 30,
        log_level: "info".to_string(),
    };
    let state = AppState::new(config);
    let app = create_app(state);

    let req = Request::builder()
        .uri("/healthz")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_root_endpoint() {
    let config = AppConfig {
        host: "127.0.0.1".to_string(),
        port: 8000,
        app_key: "dummy".to_string(),
        app_secret: "dummy".to_string(),
        cache_ttl_secs: 30,
        log_level: "info".to_string(),
    };
    let state = AppState::new(config);
    let app = create_app(state);

    let req = Request::builder()
        .uri("/")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let res = app.oneshot(req).await.unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    let body = res.into_body().collect().await.unwrap().to_bytes();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["name"], "qinping-air-monitor");
    assert_eq!(json["status"], "running");
}

#[test]
fn test_render_prometheus_metrics_success() {
    let response = Ok(QingpingDevicesResponse {
        total: 1,
        devices: vec![QingpingDevice {
            info: DeviceInfo {
                mac: "CCB5D131EE54".to_string(),
                name: "Bedroom Air Monitor".to_string(),
                version: Some("4.8.5".to_string()),
                status: Some(DeviceStatus { offline: false }),
                product: Some(ProductInfo {
                    code: Some("CGDN1".to_string()),
                    name: Some("青萍空气检测仪 Lite".to_string()),
                    en_name: Some("Qingping Air Monitor Lite".to_string()),
                }),
            },
            data: Some(SensorData {
                timestamp: Some(TimestampValue { value: 1793005200 }),
                battery: Some(NumericValue { value: 100.0 }),
                temperature: Some(NumericValue { value: 24.3 }),
                humidity: Some(NumericValue { value: 23.5 }),
                co2: Some(NumericValue { value: 551.0 }),
                pm25: Some(NumericValue { value: 3.0 }),
                pm10: Some(NumericValue { value: 3.0 }),
            }),
        }],
    });

    let rendered = render_prometheus_metrics(&response);

    assert!(rendered.contains("qingping_exporter_scrape_success 1"));
    assert!(rendered.contains("qingping_exporter_total_devices 1"));
    assert!(rendered.contains("qingping_co2_ppm{mac=\"CCB5D131EE54\",device_name=\"Bedroom Air Monitor\",product=\"CGDN1\"} 551"));
    assert!(rendered.contains("qingping_pm25_ug_m3{mac=\"CCB5D131EE54\",device_name=\"Bedroom Air Monitor\",product=\"CGDN1\"} 3"));
    assert!(rendered.contains("qingping_pm10_ug_m3{mac=\"CCB5D131EE54\",device_name=\"Bedroom Air Monitor\",product=\"CGDN1\"} 3"));
    assert!(rendered.contains("qingping_temperature_celsius{mac=\"CCB5D131EE54\",device_name=\"Bedroom Air Monitor\",product=\"CGDN1\"} 24.3"));
    assert!(rendered.contains("qingping_humidity_percent{mac=\"CCB5D131EE54\",device_name=\"Bedroom Air Monitor\",product=\"CGDN1\"} 23.5"));
    assert!(rendered.contains("qingping_battery_percent{mac=\"CCB5D131EE54\",device_name=\"Bedroom Air Monitor\",product=\"CGDN1\"} 100"));
    assert!(rendered.contains("qingping_device_online{mac=\"CCB5D131EE54\",device_name=\"Bedroom Air Monitor\",product=\"CGDN1\"} 1"));
    assert!(rendered.contains("qingping_device_last_report_timestamp_seconds{mac=\"CCB5D131EE54\",device_name=\"Bedroom Air Monitor\",product=\"CGDN1\"} 1793005200"));
}

#[test]
fn test_render_prometheus_metrics_failure() {
    let response: Result<QingpingDevicesResponse, String> = Err("Upstream timeout".to_string());
    let rendered = render_prometheus_metrics(&response);

    assert!(rendered.contains("qingping_exporter_scrape_success 0"));
}
