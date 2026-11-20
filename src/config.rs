use serde::Deserialize;
use std::collections::HashMap;

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
        Self::parse(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse config file '{}': {}", path, e))
    }

    /// Parse config from a YAML string, resolving `<<` merge keys.
    pub fn parse(content: &str) -> Result<Self, serde_yaml::Error> {
        // serde_yaml doesn't apply merge keys when deserialising directly
        let mut value: serde_yaml::Value = serde_yaml::from_str(content)?;
        value.apply_merge()?;
        serde_yaml::from_value(value)
    }

    pub fn find_provider(
        &self,
        host: &str,
        path: Option<&str>,
        username: &str,
    ) -> Option<&ProviderEntry> {
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
    let trimmed = path.trim_matches('/');
    let repo = match trimmed.find(".git/") {
        Some(pos) => &trimmed[..pos],
        None => trimmed.strip_suffix(".git").unwrap_or(trimmed),
    };
    repo.trim_matches('/').to_lowercase()
}
