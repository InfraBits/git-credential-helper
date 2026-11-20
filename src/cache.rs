use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use serde::{Deserialize, Serialize};

const CACHE_DIR: &str = "/run/git-credential-helper";
const EXPIRY_BUFFER_SECS: u64 = 300;

/// Installation tokens are scoped to the whole installation, not a single repo,
/// so tokens are cached per installation and repos map onto their installation.
#[derive(Serialize, Deserialize, Default)]
struct CacheFile {
    #[serde(default)]
    tokens: HashMap<String, CacheEntry>,
    #[serde(default)]
    installations: HashMap<String, InstallationEntry>,
}

#[derive(Serialize, Deserialize)]
struct CacheEntry {
    token: String,
    expires_at: u64,
}

/// Expires alongside the token it was resolved with, so a stale mapping
/// (e.g. after the app is reinstalled) never outlives a usable token.
#[derive(Serialize, Deserialize)]
struct InstallationEntry {
    installation_id: u64,
    expires_at: u64,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn is_fresh(expires_at: u64) -> bool {
    now_secs() + EXPIRY_BUFFER_SECS < expires_at
}

fn token_key(application_id: u64, installation_id: u64) -> String {
    format!("{application_id}:{installation_id}")
}

fn repo_key(application_id: u64, repo: &str) -> String {
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

    fn load(&self) -> CacheFile {
        fs::read_to_string(&self.path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Returns a cached token for the repo, if its installation is known and the token is still fresh.
    pub fn get(&self, application_id: u64, repo: &str) -> Option<String> {
        let file = self.load();
        let installation = file.installations.get(&repo_key(application_id, repo))?;
        if !is_fresh(installation.expires_at) {
            return None;
        }
        self.token_from(&file, application_id, installation.installation_id)
    }

    /// Returns a cached token for the installation, if still fresh.
    pub fn get_for_installation(
        &self,
        application_id: u64,
        installation_id: u64,
    ) -> Option<String> {
        self.token_from(&self.load(), application_id, installation_id)
    }

    fn token_from(
        &self,
        file: &CacheFile,
        application_id: u64,
        installation_id: u64,
    ) -> Option<String> {
        let entry = file
            .tokens
            .get(&token_key(application_id, installation_id))?;
        is_fresh(entry.expires_at).then(|| entry.token.clone())
    }

    /// Records that `repo` belongs to `installation_id`, and optionally a new token for that installation.
    pub fn store(
        &self,
        application_id: u64,
        repo: &str,
        installation_id: u64,
        token: Option<(String, u64)>,
    ) -> Result<()> {
        let mut file = self.load();

        let now = now_secs();
        file.tokens.retain(|_, v| v.expires_at > now);
        file.installations.retain(|_, v| v.expires_at > now);

        let tkey = token_key(application_id, installation_id);
        if let Some((token, expires_at)) = token {
            file.tokens
                .insert(tkey.clone(), CacheEntry { token, expires_at });
        }
        let Some(expires_at) = file.tokens.get(&tkey).map(|e| e.expires_at) else {
            return Ok(());
        };
        file.installations.insert(
            repo_key(application_id, repo),
            InstallationEntry {
                installation_id,
                expires_at,
            },
        );

        fs::create_dir_all(&self.dir)?;
        fs::set_permissions(&self.dir, fs::Permissions::from_mode(0o700))?;

        // Per-process temp file so concurrent helpers can't interleave writes
        let tmp = format!("{}.{}.tmp", self.path, std::process::id());
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
