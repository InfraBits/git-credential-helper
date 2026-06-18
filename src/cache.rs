use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

const CACHE_DIR: &str = "/run/git-credential-helper";
const EXPIRY_BUFFER_SECS: u64 = 300;

#[derive(Serialize, Deserialize, Default)]
struct CacheFile {
    entries: HashMap<String, CacheEntry>,
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    token: String,
    expires_at: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn cache_key(application_id: u64, repo: &str) -> String {
    format!("{application_id}:{repo}")
}

pub struct TokenCache {
    dir: String,
    path: String,
}

impl Default for TokenCache {
    fn default() -> Self {
        Self::new(CACHE_DIR)
    }
}

impl TokenCache {
    pub fn new(dir: impl Into<String>) -> Self {
        let dir = dir.into();
        let path = format!("{dir}/cache.json");
        Self { dir, path }
    }

    pub fn get(&self, application_id: u64, repo: &str) -> Option<String> {
        let content = fs::read_to_string(&self.path).ok()?;
        let file: CacheFile = serde_json::from_str(&content).ok()?;
        let entry = file.entries.get(&cache_key(application_id, repo))?;
        if now_secs() + EXPIRY_BUFFER_SECS < entry.expires_at {
            Some(entry.token.clone())
        } else {
            None
        }
    }

    pub fn store(&self, application_id: u64, repo: &str, token: String, expires_at: u64) -> Result<()> {
        let mut file: CacheFile = fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        let now = now_secs();
        file.entries.retain(|_, v| v.expires_at > now);
        file.entries.insert(
            cache_key(application_id, repo),
            CacheEntry { token, expires_at },
        );

        fs::create_dir_all(&self.dir)?;
        fs::set_permissions(&self.dir, fs::Permissions::from_mode(0o700))?;

        let tmp = format!("{}.tmp", self.path);
        {
            let mut f = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&tmp)?;
            f.write_all(serde_json::to_string(&file)?.as_bytes())?;
        }
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}
