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
    cache
        .store(
            1234,
            "org/repo",
            1,
            Some(("ghs_token".into(), far_future())),
        )
        .unwrap();
    assert_eq!(cache.get(1234, "org/repo").unwrap(), "ghs_token");
}

#[test]
fn different_app_ids_are_independent() {
    let (cache, _dir) = temp_cache();
    cache
        .store(1111, "org/repo", 1, Some(("token_a".into(), far_future())))
        .unwrap();
    cache
        .store(2222, "org/repo", 1, Some(("token_b".into(), far_future())))
        .unwrap();
    assert_eq!(cache.get(1111, "org/repo").unwrap(), "token_a");
    assert_eq!(cache.get(2222, "org/repo").unwrap(), "token_b");
}

#[test]
fn different_repos_are_independent() {
    let (cache, _dir) = temp_cache();
    cache
        .store(
            1234,
            "org/repo-a",
            1,
            Some(("token_a".into(), far_future())),
        )
        .unwrap();
    cache
        .store(
            1234,
            "org/repo-b",
            2,
            Some(("token_b".into(), far_future())),
        )
        .unwrap();
    assert_eq!(cache.get(1234, "org/repo-a").unwrap(), "token_a");
    assert_eq!(cache.get(1234, "org/repo-b").unwrap(), "token_b");
}

#[test]
fn repos_in_same_installation_share_token() {
    let (cache, _dir) = temp_cache();
    cache
        .store(
            1234,
            "org/repo-a",
            42,
            Some(("shared".into(), far_future())),
        )
        .unwrap();
    assert!(cache.get(1234, "org/repo-b").is_none());
    assert_eq!(cache.get_for_installation(1234, 42).unwrap(), "shared");

    cache.store(1234, "org/repo-b", 42, None).unwrap();
    assert_eq!(cache.get(1234, "org/repo-b").unwrap(), "shared");
}

#[test]
fn new_token_for_installation_applies_to_all_its_repos() {
    let (cache, _dir) = temp_cache();
    cache
        .store(1234, "org/repo-a", 42, Some(("old".into(), far_future())))
        .unwrap();
    cache
        .store(1234, "org/repo-b", 42, Some(("new".into(), far_future())))
        .unwrap();
    assert_eq!(cache.get(1234, "org/repo-a").unwrap(), "new");
}

#[test]
fn mapping_without_cached_token_is_not_stored() {
    let (cache, _dir) = temp_cache();
    cache.store(1234, "org/repo", 42, None).unwrap();
    assert!(cache.get(1234, "org/repo").is_none());
}
