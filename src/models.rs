use serde::Serialize;

#[derive(Serialize)]
pub struct SetupResponse {
    pub status: u16,
    pub api_key: Option<String>,
    pub friendly_id: Option<String>,
    pub image_url: Option<String>,
    pub filename: Option<String>,
}

#[derive(Serialize)]
pub struct DisplayResponse {
    pub status: u16,
    pub image_url: String,
    pub filename: String,
    pub update_firmware: bool,
    pub firmware_url: Option<String>,
    pub refresh_rate: String,
    pub reset_firmware: bool,
}

#[derive(Serialize)]
pub struct DisplayError {
    pub status: u16,
    pub error: String,
}
