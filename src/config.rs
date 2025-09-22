use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseSettings {
    pub path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppSettings {
    pub setup_logo_url: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Default for LogFormat {
    fn default() -> Self {
        LogFormat::Json
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingSettings {
    #[serde(default)]
    pub format: LogFormat,
}

impl Default for LoggingSettings {
    fn default() -> Self {
        LoggingSettings {
            format: LogFormat::Json,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub database: DatabaseSettings,
    pub app: AppSettings,
    #[serde(default)]
    pub logging: LoggingSettings,
}

impl ServerConfig {
    pub fn load() -> anyhow::Result<Self> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config"))
            .build()?
            .try_deserialize::<ServerConfig>()?;

        Ok(settings)
    }
}
