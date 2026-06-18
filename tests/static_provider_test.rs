use git_credential_helper::config::StaticCredsConfig;
use git_credential_helper::providers::static_creds::get_credential;

fn make_config(username: &str, password: &str) -> StaticCredsConfig {
    StaticCredsConfig {
        username: username.to_string(),
        password: password.to_string(),
    }
}

#[test]
fn returns_username_from_config() {
    let cfg = make_config("alice", "secret");
    let cred = get_credential(&cfg);
    assert_eq!(cred.username, "alice");
}

#[test]
fn returns_password_from_config() {
    let cfg = make_config("alice", "hunter2");
    let cred = get_credential(&cfg);
    assert_eq!(cred.password, "hunter2");
}

#[test]
fn preserves_special_characters_in_password() {
    let pw = "p@ss=word:with/special&chars";
    let cfg = make_config("user", pw);
    let cred = get_credential(&cfg);
    assert_eq!(cred.password, pw);
}
