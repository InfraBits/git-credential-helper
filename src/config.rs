use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub credentials: HashMap<String, Vec<HashMap<String, ProviderEntry>>>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ProviderEntry {
    pub provider: String,
    #[serde(rename = "provider-config")]
    pub provider_config: serde_yaml::Value,
    pub users: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GithubAppConfig {
    #[serde(rename = "application-id")]
    pub application_id: u64,
    pub key: String,
}

#[derive(Debug, Deserialize)]
pub struct StaticCredsConfig {
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn load(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("Failed to read config file '{}': {}", path, e))?;
        let config: Config = serde_yaml::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file '{}': {}", path, e))?;
        Ok(config)
    }

    pub fn find_provider(&self, host: &str, path: Option<&str>, username: &str) -> Option<&ProviderEntry> {
        let entries = self.credentials.get(host)?;

        let allowed = |entry: &ProviderEntry| -> bool {
            if username == "root" {
                return true;
            }
            match &entry.users {
                None => true,
                Some(users) => users.iter().any(|u| u == username),
            }
        };

        if let Some(path) = path {
            let normalized = normalize_path(path);
            for map in entries {
                for (pattern, entry) in map {
                    if pattern != "*" && pattern == &normalized && allowed(entry) {
                        return Some(entry);
                    }
                }
            }
        }

        for map in entries {
            for (pattern, entry) in map {
                if pattern == "*" && allowed(entry) {
                    return Some(entry);
                }
            }
        }

        None
    }
}

pub fn normalize_path(path: &str) -> String {
    let stripped = path.split(".git").next().unwrap_or(path);
    stripped.trim_matches('/').to_lowercase()
}
