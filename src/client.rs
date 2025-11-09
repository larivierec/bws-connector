use anyhow::Context;
use reqwest::header::HeaderMap;
use std::path::PathBuf;

/// Build a reqwest client with optional TLS customization
pub fn build_client(insecure: bool, ca_cert: &Option<PathBuf>) -> anyhow::Result<reqwest::Client> {
    let mut client_builder = reqwest::Client::builder();
    
    if insecure {
        client_builder = client_builder.danger_accept_invalid_certs(true);
    }
    
    if let Some(ca_path) = ca_cert {
        let pem = std::fs::read(ca_path).context("reading ca cert file")?;
        let certs = reqwest::Certificate::from_pem(&pem)
            .context("failed to parse CA certificate PEM")?;
        client_builder = client_builder.add_root_certificate(certs);
    }
    
    client_builder.build().context("failed to build http client")
}

/// Build headers for Warden authentication and configuration
pub fn build_headers(
    access_token: Option<String>,
    api_url: Option<String>,
    identity_url: Option<String>,
    state_path: Option<String>,
) -> anyhow::Result<HeaderMap> {
    let mut headers = HeaderMap::new();
    
    let token = access_token
        .or_else(|| std::env::var("WARDEN_ACCESS_TOKEN").ok())
        .context("access token missing; set --access-token or WARDEN_ACCESS_TOKEN")?;
    
    headers.insert("Warden-Access-Token", token.parse()?);
    
    if let Some(u) = api_url {
        headers.insert("Warden-Api-Url", u.parse()?);
    }
    if let Some(u) = identity_url {
        headers.insert("Warden-Identity-Url", u.parse()?);
    }
    if let Some(s) = state_path {
        headers.insert("Warden-State-Path", s.parse()?);
    }
    
    Ok(headers)
}
