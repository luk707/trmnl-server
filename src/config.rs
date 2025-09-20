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
pub struct ServerConfig {
    pub database: DatabaseSettings,
    pub app: AppSettings,
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
