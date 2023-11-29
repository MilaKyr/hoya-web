use config::{Config, FileFormat};
use serde::{Deserialize, Serialize};
use std::env::var;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub application: Application,
    pub parsing_delay: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Application {
    pub host: String,
    pub port: u16,
}

/// The possible runtime environment for our application.
#[derive(Debug, Eq, PartialEq)]
pub enum Environment {
    Dev,
    Prod,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Dev => "dev",
            Environment::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "dev" => Ok(Self::Dev),
            "prod" => Ok(Self::Prod),
            other => Err(format!(
                "{other} is not a supported environment. Use either `dev` or `prod`."
            )),
        }
    }
}

pub fn get_env() -> Environment {
    let environment: Environment = var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "dev".into())
        .try_into()
        .expect("Failed to parse APP_ENVIRONMENT.");
    environment
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let environment = get_env();
    let second_source = format!("configuration/{}", environment.as_str());
    let settings = Config::builder()
        .add_source(config::File::new("configuration/base", FileFormat::Yaml))
        .add_source(config::File::new(&second_source, FileFormat::Yaml))
        .build()?;
    settings.try_deserialize::<Settings>()
}
