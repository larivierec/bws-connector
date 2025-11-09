use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about = "Bitwarden Connect CLI", long_about = None)]
pub struct Cli {
    /// Base URL for the external-secrets bitwarden-sdk-server (e.g. http://localhost:9998/rest/api/1)
    #[arg(long, default_value = "http://127.0.0.1:9998/rest/api/1")]
    pub base_url: String,

    /// Warden access token header value (or set WARDEN_ACCESS_TOKEN)
    #[arg(long)]
    pub access_token: Option<String>,

    /// Optional API URL header
    #[arg(long, default_value = "https://api.bitwarden.com")]
    pub api_url: Option<String>,

    /// Optional Identity URL header
    #[arg(long, default_value = "https://identity.bitwarden.com")]
    pub identity_url: Option<String>,

    /// Optional state path header
    #[arg(long)]
    pub state_path: Option<String>,

    /// Disable TLS certificate validation (insecure)
    #[arg(long, default_value_t = false)]
    pub insecure: bool,

    /// Path to a custom CA certificate (PEM) to trust for TLS
    #[arg(long)]
    pub ca_cert: Option<PathBuf>,

    /// Try to parse the secret's `value` field as JSON and pretty-print it
    #[arg(long, default_value_t = false)]
    pub parse_value: bool,

    /// Extract a specific field from the secret's `value` (dot or slash separated path)
    #[arg(long)]
    pub field: Option<String>,

    /// Enable verbose debugging output (prints list responses and found keys)
    #[arg(long, default_value_t = false)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Get a secret by ID
    Get { id: String },
    /// Get a secret by key (looks up the secret id via list). Optional org overrides env
    GetByKey { key: String, organization_id: Option<String> },
    /// List secrets. Optional org overrides env
    List { organization_id: Option<String> },
    /// Get secrets by IDs (comma separated)
    GetByIds { ids: String },
    /// Create a secret
    Create {
        key: String,
        value: String,
        note: Option<String>,
        project_ids: Option<String>,
    },
    /// Update a secret
    Update {
        id: String,
        key: String,
        value: String,
        note: Option<String>,
        project_ids: Option<String>,
    },
    /// Delete secrets by ids (comma separated)
    Delete { ids: String },
    /// Render placeholders from stdin or a file, replacing bws://key[/path] entries
    Render { file: Option<PathBuf> },
}
