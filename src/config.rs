use std::{collections::HashMap, net::IpAddr};

use serde::Deserialize;
use url::Url;

#[derive(Debug, Deserialize)]
pub struct OidcProvider {
    pub issuer: Url,
    pub authorize_endpoint: Url,
    pub token_endpoint: Url,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
#[non_exhaustive]
pub enum IdentityProvider {
    Oidc(OidcProvider),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub base_url: Url,
    #[serde(rename = "identity-provider")]
    pub identity_providers: HashMap<String, IdentityProvider>,
}

const CONFIG_FILE_PATH: &'static str = concat!("/etc/", env!("CARGO_BIN_NAME"), "/config.toml");

pub async fn discover() -> Config {
    let raw_config = tokio::fs::read(CONFIG_FILE_PATH)
        .await
        .expect("Unable to find config file");

    toml::from_slice(&raw_config).expect("Unable to parse config file")
}
