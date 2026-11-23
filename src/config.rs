use figment::Figment;
use figment::providers::{Env, Serialized};
use serde::{Deserialize, Serialize};

/// Settings from the environment (and `.env`), with the same names as the Python app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub app_env: String,
    pub mongodb_uri: String,
    pub mongodb_db: String,
    pub port: u16,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            app_env: "local".into(),
            mongodb_uri: "mongodb://localhost:27017".into(),
            mongodb_db: "liabilities".into(),
            port: 8000,
        }
    }
}

impl Settings {
    pub fn load() -> Result<Self, Box<figment::Error>> {
        let _ = dotenvy::dotenv();
        Figment::from(Serialized::defaults(Self::default()))
            .merge(Env::raw().only(&["app_env", "mongodb_uri", "mongodb_db", "port"]))
            .extract()
            .map_err(Box::new)
    }
}
