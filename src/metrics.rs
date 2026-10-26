use crate::client::{QingpingDevice, QingpingDevicesResponse};

fn escape_label(val: &str) -> String {
    val.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
}

pub fn render_prometheus_metrics(response: &Result<QingpingDevicesResponse, String>) -> String {
    let mut out = String::new();

    match response {
        Ok(data) => {
            out.push_str("# HELP qingping_exporter_scrape_success Whether the last Qingping API scrape succeeded\n");
            out.push_str("# TYPE qingping_exporter_scrape_success gauge\n");
            out.push_str("qingping_exporter_scrape_success 1\n\n");

            out.push_str("# HELP qingping_exporter_total_devices Total number of Qingping devices configured\n");
            out.push_str("# TYPE qingping_exporter_total_devices gauge\n");
            out.push_str(&format!("qingping_exporter_total_devices {}\n\n", data.devices.len()));

            out.push_str("# HELP qingping_co2_ppm Carbon dioxide concentration in parts per million (ppm)\n");
            out.push_str("# TYPE qingping_co2_ppm gauge\n");
            for dev in &data.devices {
                if let Some(ref d) = dev.data {
                    if let Some(ref val) = d.co2 {
                        let labels = build_labels(dev);
                        out.push_str(&format!("qingping_co2_ppm{{{}}} {}\n", labels, val.value));
                    }
                }
            }
            out.push('\n');

            out.push_str("# HELP qingping_pm25_ug_m3 PM2.5 particulate matter in micrograms per cubic meter (ug/m3)\n");
            out.push_str("# TYPE qingping_pm25_ug_m3 gauge\n");
            for dev in &data.devices {
                if let Some(ref d) = dev.data {
                    if let Some(ref val) = d.pm25 {
                        let labels = build_labels(dev);
                        out.push_str(&format!("qingping_pm25_ug_m3{{{}}} {}\n", labels, val.value));
                    }
                }
            }
            out.push('\n');

            out.push_str("# HELP qingping_pm10_ug_m3 PM10 particulate matter in micrograms per cubic meter (ug/m3)\n");
            out.push_str("# TYPE qingping_pm10_ug_m3 gauge\n");
            for dev in &data.devices {
                if let Some(ref d) = dev.data {
                    if let Some(ref val) = d.pm10 {
                        let labels = build_labels(dev);
                        out.push_str(&format!("qingping_pm10_ug_m3{{{}}} {}\n", labels, val.value));
                    }
                }
            }
            out.push('\n');

            out.push_str("# HELP qingping_temperature_celsius Temperature in degrees Celsius\n");
            out.push_str("# TYPE qingping_temperature_celsius gauge\n");
            for dev in &data.devices {
                if let Some(ref d) = dev.data {
                    if let Some(ref val) = d.temperature {
                        let labels = build_labels(dev);
                        out.push_str(&format!("qingping_temperature_celsius{{{}}} {}\n", labels, val.value));
                    }
                }
            }
            out.push('\n');

            out.push_str("# HELP qingping_humidity_percent Relative humidity percentage\n");
            out.push_str("# TYPE qingping_humidity_percent gauge\n");
            for dev in &data.devices {
                if let Some(ref d) = dev.data {
                    if let Some(ref val) = d.humidity {
                        let labels = build_labels(dev);
                        out.push_str(&format!("qingping_humidity_percent{{{}}} {}\n", labels, val.value));
                    }
                }
            }
            out.push('\n');

            out.push_str("# HELP qingping_battery_percent Battery level percentage\n");
            out.push_str("# TYPE qingping_battery_percent gauge\n");
            for dev in &data.devices {
                if let Some(ref d) = dev.data {
                    if let Some(ref val) = d.battery {
                        let labels = build_labels(dev);
                        out.push_str(&format!("qingping_battery_percent{{{}}} {}\n", labels, val.value));
                    }
                }
            }
            out.push('\n');

            out.push_str("# HELP qingping_device_online Whether device is reported online (1 = online, 0 = offline)\n");
            out.push_str("# TYPE qingping_device_online gauge\n");
            for dev in &data.devices {
                let is_online = dev.info.status.as_ref().map(|s| !s.offline).unwrap_or(true);
                let labels = build_labels(dev);
                let val = if is_online { 1 } else { 0 };
                out.push_str(&format!("qingping_device_online{{{}}} {}\n", labels, val));
            }
            out.push('\n');

            out.push_str("# HELP qingping_device_last_report_timestamp_seconds Last telemetry timestamp in unix epoch seconds\n");
            out.push_str("# TYPE qingping_device_last_report_timestamp_seconds gauge\n");
            for dev in &data.devices {
                if let Some(ref d) = dev.data {
                    if let Some(ref val) = d.timestamp {
                        let labels = build_labels(dev);
                        out.push_str(&format!("qingping_device_last_report_timestamp_seconds{{{}}} {}\n", labels, val.value));
                    }
                }
            }
        }
        Err(_) => {
            out.push_str("# HELP qingping_exporter_scrape_success Whether the last Qingping API scrape succeeded\n");
            out.push_str("# TYPE qingping_exporter_scrape_success gauge\n");
            out.push_str("qingping_exporter_scrape_success 0\n");
        }
    }

    out
}

fn build_labels(dev: &QingpingDevice) -> String {
    let mac = escape_label(&dev.info.mac);
    let name = escape_label(&dev.info.name);
    let product_code = dev.info.product.as_ref()
        .and_then(|p| p.code.as_deref())
        .unwrap_or("unknown");
    let product = escape_label(product_code);

    format!("mac=\"{}\",device_name=\"{}\",product=\"{}\"", mac, name, product)
}
