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

fn near_past() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .saturating_sub(60)
}

fn expiring_soon() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + 60
}

#[test]
fn expired_token_is_not_returned() {
    let (cache, _dir) = temp_cache();
    cache
        .store(1234, "org/repo", 1, Some(("old_token".into(), near_past())))
        .unwrap();
    assert!(cache.get(1234, "org/repo").is_none());
}

#[test]
fn token_expiring_within_buffer_is_not_returned() {
    let (cache, _dir) = temp_cache();
    cache
        .store(
            1234,
            "org/repo",
            1,
            Some(("soon_token".into(), expiring_soon())),
        )
        .unwrap();
    assert!(cache.get(1234, "org/repo").is_none());
}

#[test]
fn overwrite_updates_token() {
    let (cache, _dir) = temp_cache();
    cache
        .store(1234, "org/repo", 1, Some(("old".into(), far_future())))
        .unwrap();
    cache
        .store(1234, "org/repo", 1, Some(("new".into(), far_future())))
        .unwrap();
    assert_eq!(cache.get(1234, "org/repo").unwrap(), "new");
}

#[test]
fn expired_entries_pruned_on_store() {
    let (cache, _dir) = temp_cache();
    cache
        .store(1234, "org/stale", 1, Some(("stale".into(), near_past())))
        .unwrap();
    cache
        .store(1234, "org/fresh", 2, Some(("fresh".into(), far_future())))
        .unwrap();
    assert!(cache.get(1234, "org/stale").is_none());
    assert_eq!(cache.get(1234, "org/fresh").unwrap(), "fresh");
}
