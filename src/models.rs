use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct SetupResponse {
    pub status: u16,
    pub api_key: Option<String>,
    pub friendly_id: Option<String>,
    pub image_url: Option<String>,
    pub filename: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DisplayResponse {
    pub status: u16,
    pub image_url: String,
    pub filename: String,
    pub update_firmware: bool,
    pub firmware_url: Option<String>,
    pub refresh_rate: String,
    pub reset_firmware: bool,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub mac: Option<String>,
    pub rssi: Option<i64>,
    pub battery_voltage: Option<f64>,
    pub fw_version: Option<String>,
    pub refresh_rate: Option<i64>,
}

#[derive(Clone)]
pub struct Device {
    pub id: String,
    pub mac: Option<String>,
    pub _api_key: String,
    pub rssi: Option<i64>,
    pub battery_voltage: Option<f64>,
    pub fw_version: Option<String>,
    pub refresh_rate: Option<i64>,
    pub images: Vec<String>,
}
