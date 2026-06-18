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
  lfs.example.com:
    - '*':
        provider: static
        provider-config:
          username: user
          password: pass
"#;

#[test]
fn unknown_host_returns_none() {
    let cfg = parse(BASIC_CONFIG);
    assert!(cfg.find_provider("unknown.host", None, "alice").is_none());
}

#[test]
fn wildcard_matches_when_no_path() {
    let cfg = parse(BASIC_CONFIG);
    let entry = cfg.find_provider("github.com", None, "alice").unwrap();
    let gc: serde_yaml::Value = entry.provider_config.clone();
    assert_eq!(gc["application-id"].as_u64().unwrap(), 1234);
}

#[test]
fn specific_pattern_takes_priority_over_wildcard() {
    let cfg = parse(BASIC_CONFIG);
    let entry = cfg.find_provider("github.com", Some("org/repo"), "alice").unwrap();
    let gc: serde_yaml::Value = entry.provider_config.clone();
    assert_eq!(gc["application-id"].as_u64().unwrap(), 4567);
}

#[test]
fn unmatched_path_falls_back_to_wildcard() {
    let cfg = parse(BASIC_CONFIG);
    let entry = cfg.find_provider("github.com", Some("other/repo"), "alice").unwrap();
    let gc: serde_yaml::Value = entry.provider_config.clone();
    assert_eq!(gc["application-id"].as_u64().unwrap(), 1234);
}

#[test]
fn git_suffix_path_matches_specific_pattern() {
    let cfg = parse(BASIC_CONFIG);
    let entry = cfg.find_provider("github.com", Some("org/repo.git"), "alice").unwrap();
    let gc: serde_yaml::Value = entry.provider_config.clone();
    assert_eq!(gc["application-id"].as_u64().unwrap(), 4567);
}

#[test]
fn static_provider_matched_by_host() {
    let cfg = parse(BASIC_CONFIG);
    let entry = cfg.find_provider("lfs.example.com", None, "alice").unwrap();
    assert_eq!(entry.provider, "static");
}
