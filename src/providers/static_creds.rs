use super::Credential;
use crate::config::StaticCredsConfig;

pub fn get_credential(config: &StaticCredsConfig) -> Credential {
    Credential {
        username: config.username.clone(),
        password: config.password.clone(),
    }
}
