use crate::config::StaticCredsConfig;
use super::Credential;

pub fn get_credential(config: &StaticCredsConfig) -> Credential {
    Credential {
        username: config.username.clone(),
        password: config.password.clone(),
    }
}
