use git_credential_helper::config::Config;

fn parse(yaml: &str) -> Config {
    serde_yaml::from_str(yaml).expect("valid yaml")
}

const BASIC_CONFIG: &str = r#"
credentials:
  github.com:
    - 'org/repo':
        provider: github-application
        provider-config:
          application-id: 4567
          key: /keys/repo.key
    - '*':
        provider: github-application
        provider-config:
          application-id: 1234
          key: /keys/fallback.key
"#;

const USERS_CONFIG: &str = r#"
credentials:
  github.com:
    - 'org/restricted':
        provider: github-application
        users:
          - alice
        provider-config:
          application-id: 111
          key: /keys/restricted.key
    - '*':
        provider: github-application
        provider-config:
          application-id: 222
          key: /keys/open.key
  lfs.example.com:
    - '*':
        provider: static
        users:
          - alice
          - bob
        provider-config:
          username: user
          password: pass
"#;

#[test]
fn listed_user_can_access_restricted_entry() {
    let cfg = parse(USERS_CONFIG);
    let entry = cfg.find_provider("github.com", Some("org/restricted"), "alice").unwrap();
    let gc: serde_yaml::Value = entry.provider_config.clone();
    assert_eq!(gc["application-id"].as_u64().unwrap(), 111);
}

#[test]
fn unlisted_user_skips_restricted_entry_falls_back_to_wildcard() {
    let cfg = parse(USERS_CONFIG);
    let entry = cfg.find_provider("github.com", Some("org/restricted"), "charlie").unwrap();
    let gc: serde_yaml::Value = entry.provider_config.clone();
    assert_eq!(gc["application-id"].as_u64().unwrap(), 222);
}

#[test]
fn root_bypasses_user_restriction() {
    let cfg = parse(USERS_CONFIG);
    let entry = cfg.find_provider("github.com", Some("org/restricted"), "root").unwrap();
    let gc: serde_yaml::Value = entry.provider_config.clone();
    assert_eq!(gc["application-id"].as_u64().unwrap(), 111);
}

#[test]
fn unlisted_user_gets_none_when_only_restricted_wildcard() {
    let cfg = parse(USERS_CONFIG);
    assert!(cfg.find_provider("lfs.example.com", None, "charlie").is_none());
}

#[test]
fn listed_user_gets_static_wildcard() {
    let cfg = parse(USERS_CONFIG);
    let entry = cfg.find_provider("lfs.example.com", None, "bob").unwrap();
    assert_eq!(entry.provider, "static");
}

#[test]
fn no_users_field_allows_any_user() {
    let cfg = parse(BASIC_CONFIG);
    let entry = cfg.find_provider("github.com", Some("other/repo"), "anyone").unwrap();
    let gc: serde_yaml::Value = entry.provider_config.clone();
    assert_eq!(gc["application-id"].as_u64().unwrap(), 1234);
}
