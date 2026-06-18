use git_credential_helper::config::normalize_path;

#[test]
fn normalize_strips_git_suffix() {
    assert_eq!(normalize_path("org/repo.git"), "org/repo");
}

#[test]
fn normalize_lowercases() {
    assert_eq!(normalize_path("Org/Repo"), "org/repo");
}

#[test]
fn normalize_strips_leading_slash() {
    assert_eq!(normalize_path("/org/repo"), "org/repo");
}

#[test]
fn normalize_strips_git_then_lowercases() {
    assert_eq!(normalize_path("/Org/Repo.git"), "org/repo");
}

#[test]
fn normalize_plain_path_unchanged() {
    assert_eq!(normalize_path("org/repo"), "org/repo");
}
