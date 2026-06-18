use anyhow::Context;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::cache::TokenCache;
use crate::config::{normalize_path, GithubAppConfig};
use super::Credential;

#[derive(Serialize)]
struct JwtClaims {
    iat: i64,
    exp: i64,
    iss: u64,
}

#[derive(Deserialize)]
struct InstallationResponse {
    id: u64,
}

#[derive(Deserialize)]
struct AccessTokenResponse {
    token: String,
    expires_at: Option<String>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn parse_expires_at(s: &str) -> u64 {
    parse_rfc3339(s).unwrap_or_else(|| now_secs() + 3600)
}

fn parse_rfc3339(s: &str) -> Option<u64> {
    let s = s.strip_suffix('Z')?;
    let (date, time) = s.split_once('T')?;
    let mut dp = date.splitn(3, '-');
    let year: u64 = dp.next()?.parse().ok()?;
    let month: u64 = dp.next()?.parse().ok()?;
    let day: u64 = dp.next()?.parse().ok()?;
    let mut tp = time.splitn(3, ':');
    let hour: u64 = tp.next()?.parse().ok()?;
    let min: u64 = tp.next()?.parse().ok()?;
    let sec: u64 = tp.next()?.parse().ok()?;

    let y = if month <= 2 { year - 1 } else { year };
    let era = y / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if month > 2 { month - 3 } else { month + 9 }) + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146097 + doe;
    let unix_days = days.checked_sub(719468)?;
    Some(unix_days * 86400 + hour * 3600 + min * 60 + sec)
}

fn generate_jwt(app_id: u64, pem_path: &str) -> anyhow::Result<String> {
    let pem = std::fs::read(pem_path)
        .with_context(|| format!("Failed to read key file '{}'", pem_path))?;

    let encoding_key = EncodingKey::from_rsa_pem(&pem)
        .with_context(|| format!("Failed to parse RSA PEM key from '{}'", pem_path))?;

    let now = now_secs() as i64;
    let claims = JwtClaims {
        iat: now - 60,
        exp: now + 600,
        iss: app_id,
    };

    let token = encode(&Header::new(Algorithm::RS256), &claims, &encoding_key)
        .context("Failed to encode JWT")?;
    Ok(token)
}

async fn find_installation_id(
    client: &reqwest::Client,
    bearer_token: &str,
    repository: &str,
) -> anyhow::Result<Option<u64>> {
    let response = client
        .get(format!("https://api.github.com/repos/{repository}/installation"))
        .header("Authorization", format!("Bearer {bearer_token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "git-credential-helper")
        .send()
        .await
        .context("Failed to send request to GitHub API")?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API error {status}: {body}");
    }

    let installation: InstallationResponse = response
        .json()
        .await
        .context("Failed to parse installation response")?;
    Ok(Some(installation.id))
}

async fn fetch_access_token(
    client: &reqwest::Client,
    bearer_token: &str,
    installation_id: u64,
) -> anyhow::Result<AccessTokenResponse> {
    let response = client
        .post(format!(
            "https://api.github.com/app/installations/{installation_id}/access_tokens"
        ))
        .header("Authorization", format!("Bearer {bearer_token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "git-credential-helper")
        .send()
        .await
        .context("Failed to send access token request")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("GitHub API error {status}: {body}");
    }

    response
        .json()
        .await
        .context("Failed to parse access token response")
}

pub async fn get_credential(config: &GithubAppConfig, path: &str) -> anyhow::Result<Option<Credential>> {
    let normalized = normalize_path(path);
    let cache = TokenCache::default();

    if let Some(token) = cache.get(config.application_id, &normalized) {
        return Ok(Some(Credential {
            username: "oauth2".to_string(),
            password: token,
        }));
    }

    let bearer_token = generate_jwt(config.application_id, &config.key)?;
    let client = reqwest::Client::new();

    let Some(installation_id) = find_installation_id(&client, &bearer_token, &normalized).await? else {
        return Ok(None);
    };

    let token_response = fetch_access_token(&client, &bearer_token, installation_id).await?;
    let expires_at = token_response
        .expires_at
        .as_deref()
        .map(parse_expires_at)
        .unwrap_or_else(|| now_secs() + 3600);

    let _ = cache.store(
        config.application_id,
        &normalized,
        token_response.token.clone(),
        expires_at,
    );

    Ok(Some(Credential {
        username: "oauth2".to_string(),
        password: token_response.token,
    }))
}
