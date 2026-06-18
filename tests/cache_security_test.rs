use std::os::unix::fs::PermissionsExt;
use git_credential_helper::cache::TokenCache;

fn temp_cache() -> (TokenCache, tempfile::TempDir) {
    let dir = tempfile::tempdir().expect("temp dir");
    let cache = TokenCache::new(dir.path().to_str().unwrap());
    (cache, dir)
}

fn far_future() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 3600
}

#[test]
fn cache_file_has_mode_0600() {
    let (cache, _dir) = temp_cache();
    cache.store(1234, "org/repo", "token".into(), far_future()).unwrap();
    let path = format!("{}/cache.json", _dir.path().to_str().unwrap());
    let mode = std::fs::metadata(path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600, "cache file should be mode 0600, got {mode:o}");
}

#[test]
fn cache_dir_has_mode_0700() {
    let (cache, dir) = temp_cache();
    cache.store(1234, "org/repo", "token".into(), far_future()).unwrap();
    let mode = std::fs::metadata(dir.path()).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o700, "cache dir should be mode 0700, got {mode:o}");
}
