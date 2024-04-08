use crate::errors::ConfigurationError;
use config::{Config, FileFormat};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::env::var;
use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Settings {
    pub application: Application,
    pub database: DatabaseSettings,
    pub parsing_delay: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Application {
    pub host: String,
    pub port: u16,
}
#[serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DatabaseSettings {
    #[serde_as(as = "DisplayFromStr")]
    pub db_type: DatabaseType,
    pub file_path: Option<String>,
    pub protocol: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub user: Option<String>,
    pub password: Option<String>,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub enum DatabaseType {
    InMemory,
    Relational,
}

impl Display for DatabaseType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::InMemory => write!(f, "in_memory"),
            DatabaseType::Relational => write!(f, "relational"),
        }
    }
}

impl FromStr for DatabaseType {
    type Err = ConfigurationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "relational" => Ok(DatabaseType::Relational),
            "in_memory" => Ok(DatabaseType::InMemory),
            &_ => Err(ConfigurationError::UnknownDatabaseType),
        }
    }
}

impl DatabaseSettings {
    pub fn check_if_valid(&self) -> Result<(), ConfigurationError> {
        match self.db_type {
            DatabaseType::InMemory => match &self.file_path {
                None => return Err(ConfigurationError::DataFileNotFound),
                Some(path) => {
                    if !Path::new(path).is_file() {
                        return Err(ConfigurationError::DataFileNotFound);
                    }
                }
            },
            DatabaseType::Relational => {
                if self.protocol.is_none()
                    || self.host.is_none()
                    || self.name.is_none()
                    || self.port.is_none()
                    || self.user.is_none()
                    || self.password.is_none()
                {
                    return Err(ConfigurationError::MissingDatabaseSettings);
                }
            }
        }
        Ok(())
    }

    pub fn path_unchecked(&self) -> String {
        self.file_path.to_owned().unwrap()
    }

    pub fn relational_connection_unchecked(&self) -> String {
        format!(
            "{}://{}:{}@{}:{}/{}",
            self.protocol.to_owned().unwrap(),
            self.user.to_owned().unwrap(),
            self.password.to_owned().unwrap(),
            self.host.to_owned().unwrap(),
            self.port.to_owned().unwrap(),
            self.name.to_owned().unwrap(),
        )
    }
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
