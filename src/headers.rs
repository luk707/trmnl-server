use axum::http::HeaderName;

pub const HEADER_MAC: HeaderName = HeaderName::from_static("id");
pub const HEADER_ACCESS_TOKEN: HeaderName = HeaderName::from_static("access-token");
pub const HEADER_FW_VERSION: HeaderName = HeaderName::from_static("fw-version");
pub const HEADER_BATTERY_VOLTAGE: HeaderName = HeaderName::from_static("battery-voltage");
pub const HEADER_REFRESH_RATE: HeaderName = HeaderName::from_static("refresh-rate");
pub const HEADER_RSSI: HeaderName = HeaderName::from_static("rssi");
