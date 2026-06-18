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
fn miss_on_empty_cache() {
    let (cache, _dir) = temp_cache();
    assert!(cache.get(1234, "org/repo").is_none());
}

#[test]
fn stores_and_retrieves_token() {
    let (cache, _dir) = temp_cache();
    cache.store(1234, "org/repo", "ghs_token".into(), far_future()).unwrap();
    assert_eq!(cache.get(1234, "org/repo").unwrap(), "ghs_token");
}

#[test]
fn different_app_ids_are_independent() {
    let (cache, _dir) = temp_cache();
    cache.store(1111, "org/repo", "token_a".into(), far_future()).unwrap();
    cache.store(2222, "org/repo", "token_b".into(), far_future()).unwrap();
    assert_eq!(cache.get(1111, "org/repo").unwrap(), "token_a");
    assert_eq!(cache.get(2222, "org/repo").unwrap(), "token_b");
}

#[test]
fn different_repos_are_independent() {
    let (cache, _dir) = temp_cache();
    cache.store(1234, "org/repo-a", "token_a".into(), far_future()).unwrap();
    cache.store(1234, "org/repo-b", "token_b".into(), far_future()).unwrap();
    assert_eq!(cache.get(1234, "org/repo-a").unwrap(), "token_a");
    assert_eq!(cache.get(1234, "org/repo-b").unwrap(), "token_b");
}
