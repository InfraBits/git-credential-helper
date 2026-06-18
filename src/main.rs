use std::collections::HashMap;
use std::io::{self, BufRead};
use std::os::unix::process::CommandExt;

use anyhow::Context;
use clap::Parser;
use git_credential_helper::config::{Config, GithubAppConfig, StaticCredsConfig};
use git_credential_helper::providers;

#[derive(Parser)]
#[command(name = "git-credential-helper")]
struct Cli {
    #[arg(long, default_value = "/etc/git-credential-helper/config.yaml")]
    config: String,

    operation: Option<String>,
}

fn resolve_username() -> String {
    if let Ok(sudo_user) = std::env::var("SUDO_USER") {
        if !sudo_user.is_empty() {
            return sudo_user;
        }
    }
    let uid = nix::unistd::getuid();
    match nix::unistd::User::from_uid(uid) {
        Ok(Some(user)) => user.name,
        _ => "unknown".to_string(),
    }
}

fn read_git_credential_input() -> anyhow::Result<HashMap<String, String>> {
    let stdin = io::stdin();
    let mut payload = HashMap::new();

    for line in stdin.lock().lines() {
        let line = line.context("Failed to read from stdin")?;
        let line = line.trim().to_string();

        if line.is_empty() {
            break;
        }

        if let Some(pos) = line.find('=') {
            let key = line[..pos].to_string();
            let value = line[pos + 1..].to_string();
            payload.insert(key, value);
        }
    }

    Ok(payload)
}

#[tokio::main]
async fn main() {
    if !nix::unistd::getuid().is_root() {
        let err = std::process::Command::new("sudo")
            .args(std::env::args())
            .exec();
        eprintln!("Failed to re-exec with sudo: {err}");
        std::process::exit(1);
    }

    if let Err(e) = run().await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.operation.as_deref() {
        Some("get") => {}
        _ => return Ok(()),
    }

    let config = Config::load(&cli.config)?;
    let username = resolve_username();

    let payload = read_git_credential_input()?;

    let host = match payload.get("host") {
        Some(h) => h.as_str(),
        None => {
            anyhow::bail!("Missing host in payload: {:?}", payload);
        }
    };

    let path = payload.get("path").map(|s| s.as_str());

    let provider_entry = match config.find_provider(host, path, &username) {
        Some(entry) => entry.clone(),
        None => return Ok(()),
    };

    match provider_entry.provider.as_str() {
        "github-application" => {
            let path = match path {
                Some(p) => p,
                None => anyhow::bail!("Missing path in payload for github-application provider"),
            };

            let gh_config: GithubAppConfig = serde_yaml::from_value(provider_entry.provider_config)
                .context("Failed to parse github-application provider config")?;

            if let Some(cred) = providers::github_app::get_credential(&gh_config, path).await? {
                println!("username={}", cred.username);
                println!("password={}", cred.password);
            }
        }
        "static" => {
            let static_config: StaticCredsConfig =
                serde_yaml::from_value(provider_entry.provider_config)
                    .context("Failed to parse static provider config")?;

            let cred = providers::static_creds::get_credential(&static_config);
            println!("username={}", cred.username);
            println!("password={}", cred.password);
        }
        unknown => {
            anyhow::bail!("Unknown provider: '{}'", unknown);
        }
    }

    Ok(())
}
