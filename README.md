# git-credential-helper

A git credential helper that provisions credentials from multiple providers, configured via a YAML file. Supports GitHub App installation tokens and static credentials, with per-entry user access control.

## Providers

### `github-application`

Generates a short-lived GitHub App installation token using RS256 JWT authentication. The token is returned as `username=oauth2 / password=<token>`.

Requires:
- `application-id` — GitHub App ID
- `key` — path to the app's RSA private key (PEM format)

### `static`

Returns hardcoded credentials from the config.

Requires:
- `username`
- `password`

## Configuration

Default config path: `/etc/git-credential-helper/config.yaml` (override with `--config`).

Credentials are matched by host, then by repository path. Specific `org/repo` patterns take priority over the `*` wildcard. Entries are checked in order.

```yaml
credentials:
  github.com:
    - 'org/repo':
        provider: github-application
        users:           # optional — omit to allow all users
          - alice
          - bob
        provider-config:
          application-id: 4567
          key: /etc/git-credential-helper/keys/repo-specific-key.key

    - '*':
        provider: github-application
        provider-config:
          application-id: 1234
          key: /etc/git-credential-helper/keys/github-fallback.key

  lfs-shimmy.infrabits.nl:
    - '*':
        provider: static
        users:
          - alice
        provider-config:
          username: example
          password: secret
```

### Access control

If `users` is set on an entry, only listed users can retrieve those credentials — non-listed users get no credentials (same as a missing entry). `root` is always allowed regardless.

The binary resolves the calling user from `SUDO_USER` (set by sudo) and falls back to the current process uid. Since the binary is typically invoked via `sudo`, configure sudoers accordingly:

```
%git-users ALL=(root) NOPASSWD: /usr/local/bin/git-credential-helper
```

## Git configuration

Configure as a credential helper scoped to a specific host:

```ini
[credential "https://github.com"]
    helper = "!sudo /usr/local/bin/git-credential-helper"

[credential "https://lfs-shimmy.infrabits.nl"]
    helper = "!sudo /usr/local/bin/git-credential-helper"
```

## Installation

Download a pre-built binary from the [releases page](../../releases) (`x86_64` and `aarch64` musl builds are provided), or build from source:

```sh
cargo build --release
install -m 755 target/release/git-credential-helper /usr/local/bin/
```

Place your config at `/etc/git-credential-helper/config.yaml` and your key files wherever the config references them (ensure they are readable only by root).
