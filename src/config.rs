use std::path::Path;

fn config_path() -> String {
    std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string())
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub admin_password: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            admin_password: "admin123".into(),
            port: 10119,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path_str = config_path();
        let path = Path::new(&path_str);
        if !path.exists() {
            let cfg = Config::default();
            cfg.save();
            return cfg;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_else(|_| {
            eprintln!("Warning: config.toml parse error, using defaults");
            Config::default()
        })
    }

    pub fn save(&self) {
        if let Ok(content) = toml::to_string_pretty(self) {
            let _ = std::fs::write(config_path(), content);
        }
    }
}
