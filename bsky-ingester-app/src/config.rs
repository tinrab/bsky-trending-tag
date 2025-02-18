use std::env;
use std::path::Path;
use std::sync::OnceLock;

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub distribution: DistributionConfig,
    pub broker: BrokerConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistributionConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum BrokerConfig {
    Memory,
    Nats(NatsBrokerConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsBrokerConfig {
    pub address: String,
    pub user: Option<String>,
    pub password: Option<String>,
    pub subject_prefix: Option<String>,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

const CONFIG_PATH_ENV: &str = "APP_INGESTER_CONFIG_PATH";
const CONFIG_PATH_DEFAULT: &str = "config";
const ENV_PREFIX: &str = "APP_INGESTER";

const DISTRIBUTION_VERSION_KEY: &str = "distribution.version";

impl AppConfig {
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<AppConfig> = OnceLock::new();
        INSTANCE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> Result<Self, ConfigError> {
        let mut config_builder =
            Config::builder().set_default(DISTRIBUTION_VERSION_KEY, VERSION)?;

        let config_path = env::var(CONFIG_PATH_ENV).unwrap_or(CONFIG_PATH_DEFAULT.to_string());

        // Initial "default" configuration file
        let default_path = Path::new(&config_path).join("default");
        config_builder = config_builder.add_source(File::with_name(default_path.to_str().unwrap()));

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        let local_path = Path::new(&config_path).join("local");
        config_builder = config_builder
            .add_source(File::with_name(local_path.to_str().unwrap()).required(false));

        // Add in settings from the environment (with a prefix of APP)
        config_builder =
            config_builder.add_source(Environment::with_prefix(ENV_PREFIX).separator("__"));

        // Set derived properties
        // let config = config_builder.build()?;
        // let mut config_builder = Config::builder();

        let config = config_builder
            // .add_source(config)
            .build()?
            .try_deserialize()?;
        Ok(config)
    }
}
