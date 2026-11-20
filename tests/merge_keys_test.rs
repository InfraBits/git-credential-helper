use git_credential_helper::config::Config;

const MERGE_CONFIG: &str = r#"
defaults:
  github_provider: &github_provider
    provider: github-application
    provider-config:
      application-id: 1234
      key: /keys/default.key
  lfs_provider: &lfs_provider
    provider: static
    provider-config:
      username: user
      password: pass

credentials:
  github.com:
    - 'org/repo':
        users: ['alice']
        <<: *github_provider
    - 'org/other':
        <<: *github_provider
        provider-config:
          application-id: 4567
          key: /keys/other.key
  lfs.example.com:
    - '*':
        <<: *lfs_provider
"#;

#[test]
fn merge_keys_are_applied() {
    let cfg = Config::parse(MERGE_CONFIG).unwrap();
    let entry = cfg
        .find_provider("github.com", Some("org/repo"), "alice")
        .unwrap();
    assert_eq!(entry.provider, "github-application");
    assert_eq!(
        entry.provider_config["application-id"].as_u64().unwrap(),
        1234
    );
    assert_eq!(entry.users, Some(vec!["alice".to_string()]));

    let entry = cfg.find_provider("lfs.example.com", None, "bob").unwrap();
    assert_eq!(entry.provider, "static");
    assert_eq!(entry.provider_config["username"].as_str().unwrap(), "user");
}

#[test]
fn explicit_keys_override_merged_keys() {
    let cfg = Config::parse(MERGE_CONFIG).unwrap();
    let entry = cfg
        .find_provider("github.com", Some("org/other"), "alice")
        .unwrap();
    assert_eq!(entry.provider, "github-application");
    assert_eq!(
        entry.provider_config["application-id"].as_u64().unwrap(),
        4567
    );
}

#[test]
fn config_without_merge_keys_still_parses() {
    let cfg = Config::parse(
        r#"
credentials:
  github.com:
    - '*':
        provider: github-application
        provider-config:
          application-id: 1234
          key: /keys/fallback.key
"#,
    )
    .unwrap();
    assert!(cfg.find_provider("github.com", None, "alice").is_some());
}
