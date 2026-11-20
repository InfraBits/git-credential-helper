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

#[test]
fn normalize_keeps_git_inside_repo_name() {
    assert_eq!(normalize_path("org/org.github.io"), "org/org.github.io");
}

#[test]
fn normalize_strips_git_suffix_from_github_io_repo() {
    assert_eq!(normalize_path("Org/Org.github.io.git"), "org/org.github.io");
}

#[test]
fn normalize_strips_git_lfs_suffix() {
    assert_eq!(normalize_path("org/repo.git/info/lfs"), "org/repo");
}

#[test]
fn normalize_strips_trailing_slash() {
    assert_eq!(normalize_path("org/repo.git/"), "org/repo");
}
