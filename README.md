# Qingping Air Monitor Exporter (`qinping-air-monitor`)

Prometheus metrics exporter for **Qingping Air Monitor Lite** (and other Qingping air quality monitors) connecting via the official [Qingping Cloud OpenAPI](https://developer.qingping.co/).

## Features

- **Sensors telemetry**:
  - `qingping_co2_ppm`: Carbon dioxide (CO₂) concentration in ppm.
  - `qingping_pm25_ug_m3`: PM2.5 concentration in µg/m³.
  - `qingping_pm10_ug_m3`: PM10 concentration in µg/m³.
  - `qingping_temperature_celsius`: Ambient temperature in °C.
  - `qingping_humidity_percent`: Relative humidity in %.
  - `qingping_battery_percent`: Device battery level in %.
  - `qingping_device_online`: Device online status (1 = online, 0 = offline).
  - `qingping_device_last_report_timestamp_seconds`: Timestamp of last sensor report.
  - `qingping_exporter_scrape_success`: Scrape status health indicator.
- **In-memory caching**: Built-in configurable TTL cache (`METRICS_CACHE_TTL_SECS`) to prevent hitting Qingping API rate limits during high-frequency Prometheus scrapes.
- **Production-grade Docker image**: Multi-stage musl build packaged into a minimal `scratch` image running as non-root user (UID 10001).
- **Kubernetes / Helm**: Ready-to-deploy chart in `helm/` with `existingSecret` pattern and Prometheus Operator `ServiceMonitor`.
- **Automated CI/CD**: Forgejo Actions workflow for Docker Buildx and pushing images to `zot.xinfra.ru`.
- **Versioning**: Integrated `bump-my-version` support.

---

## Configuration

Configuration is loaded from environment variables (or `.env` file via `dotenvy`):

| Variable | Description | Default |
|---|---|---|
| `QINGPING_APP_KEY` | Qingping Developer App Key | **Required** |
| `QINGPING_APP_SECRET` | Qingping Developer App Secret | **Required** |
| `PORT` | HTTP server port | `8000` |
| `HOST` | HTTP listen address | `0.0.0.0` |
| `METRICS_CACHE_TTL_SECS` | Cache duration for device metrics | `30` |
| `LOG_LEVEL` | Tracing log level (`info`, `debug`, `warn`) | `info` |

---

## Quick Start (Local)

1. Copy `.env.example` to `.env` and fill in your credentials:
   ```bash
   cp .env.example .env
   ```

2. Run the exporter:
   ```bash
   cargo run
   ```

3. Query endpoints:
   ```bash
   curl http://localhost:8000/healthz
   curl http://localhost:8000/metrics
   ```

---

## Kubernetes Deployment (Helm)

The chart is located in `helm/`.

### Using `existingSecret` (Recommended for GitOps / FluxCD)

Create a Kubernetes Secret (e.g. via SealedSecrets, ExternalSecrets, or SOPS):

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: qingping-creds
type: Opaque
stringData:
  QINGPING_APP_KEY: "your_app_key"
  QINGPING_APP_SECRET: "your_app_secret"
```

Then install the chart referencing the existing secret:

```bash
helm install qinping-air-monitor helm/ \
  --set existingSecret=qingping-creds
```

### Prometheus Operator ServiceMonitor & Grafana Dashboard

The chart automatically creates a `ServiceMonitor` and imports the Grafana dashboard for `kube-prometheus-stack`:

```yaml
serviceMonitor:
  enabled: true
  interval: "30s"
  scrapeTimeout: "10s"

grafanaDashboard:
  enabled: true
  labels:
    grafana_dashboard: "1"
```

The dashboard will be automatically discovered by the `kube-prometheus-stack` Grafana sidecar and imported into the **Environment** folder.


---

## Versioning

Use `bump-my-version` via `uvx`:

```bash
# Check current version and configuration
uvx bump-my-version show

# Check possible bumps
uvx bump-my-version show-bump

# Bump patch (0.1.0 -> 0.1.1)
uvx bump-my-version bump patch

# Bump minor (0.1.0 -> 0.2.0)
uvx bump-my-version bump minor
```
