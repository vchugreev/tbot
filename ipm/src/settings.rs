use config::{Config, ConfigError, File};
use serde::Deserialize;

const CONFIG_DEFAULT_FILE: &str = "default.yaml";
const CONFIGS_DEFAULT_PATH: &str = "./configs/";

#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    pub level: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Tinkoff {
    pub ws: String,
    pub figis: Vec<String>,
    pub token: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Grpc {
    pub addr: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Client {
    pub tinkoff: Tinkoff,
    pub pr: Grpc,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Env {
    Development,
    Testing,
    Production,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Settings {
    pub log: Log,
    pub env: Env,
    pub server: Grpc,
    pub client: Client,
}

// Пример отсюда: https://blog.logrocket.com/configuration-management-in-rust-web-services/
impl Settings {
    pub fn new(configs_path: Option<&str>) -> Result<Self, ConfigError> {
        let path = match configs_path {
            Some(p) => p,
            None => CONFIGS_DEFAULT_PATH,
        };

        let env = std::env::var("RUN_ENV").unwrap_or_else(|_| "Development".into());
        let mut cfg = Config::new();
        cfg.set("env", env.clone())?;

        let file_name = env.to_lowercase();

        cfg.merge(File::with_name(&format!("{}{}", path, CONFIG_DEFAULT_FILE)))?;
        cfg.merge(File::with_name(&format!("{}{}", path, file_name)))?;

        cfg.try_into()
    }
}
