mod cli;
mod client;
mod models;
mod output;
mod render;

use anyhow::Context;
use clap::Parser;

use cli::{Cli, Commands};
use models::*;
use output::print_response_with_parsed_value;
use render::{read_input, render_template};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    
    // Organization id is required via env var
    let org_id = std::env::var("WARDEN_ORGANIZATION_ID")
        .context("WARDEN_ORGANIZATION_ID must be set")?;
    
    // Build client and headers
    let client = client::build_client(cli.insecure, &cli.ca_cert)?;
    let headers = client::build_headers(
        cli.access_token.clone(),
        cli.api_url.clone(),
        cli.identity_url.clone(),
        cli.state_path.clone(),
    )?;

    let base = cli.base_url.trim_end_matches('/').to_string();

    match cli.command {
        Commands::Get { id } => {
            let req = SecretGetRequest { id };
            let url = format!("{}/secret", base);
            let resp = client
                .get(&url)
                .headers(headers)
                .json(&req)
                .send()
                .await
                .context("request failed")?;
            let txt = resp.text().await?;
            print_response_with_parsed_value(&txt, cli.parse_value, cli.field.as_deref())?;
        }
    Commands::GetByKey { key, organization_id } => {
            // call list endpoint to find id by key
            let url = format!("{}/secrets", base);
            let org_to_use = organization_id.as_ref().map(|s| s.as_str()).unwrap_or(&org_id);
            let req_builder = client.get(&url).headers(headers.clone());
            let resp = req_builder.json(&serde_json::json!({"OrganizationID": org_to_use})).send().await.context("request failed")?;
            let txt = resp.text().await?;
            if cli.verbose {
                eprintln!("raw list response: {}", txt);
            }
            let list: ListResponse = serde_json::from_str(&txt).context("failed to parse list response")?;
            if cli.verbose {
                let keys: Vec<String> = list.data.iter().map(|i| i.key.clone()).collect();
                eprintln!("found keys: {:?}", keys);
            }
            let found = list.data.into_iter().find(|i| i.key == key).ok_or_else(|| anyhow::anyhow!("secret with key not found"))?;

            // fetch the secret by id
            let get_req = SecretGetRequest { id: found.id };
            let url = format!("{}/secret", base);
            let resp = client
                .get(&url)
                .headers(headers)
                .json(&get_req)
                .send()
                .await
                .context("request failed")?;
            let txt = resp.text().await?;
            print_response_with_parsed_value(&txt, cli.parse_value, cli.field.as_deref())?;
        }
        Commands::List { organization_id } => {
            let url = format!("{}/secrets", base);
            let org_to_use = organization_id.as_ref().map(|s| s.as_str()).unwrap_or(&org_id);
            let resp = client.get(&url).headers(headers).json(&serde_json::json!({"OrganizationID": org_to_use})).send().await.context("request failed")?;
            let txt = resp.text().await?;
            if cli.verbose {
                eprintln!("raw list response: {}", txt);
                let list: ListResponse = serde_json::from_str(&txt).unwrap_or(ListResponse{data: vec![]});
                let keys: Vec<String> = list.data.iter().map(|i| i.key.clone()).collect();
                eprintln!("found keys: {:?}", keys);
            }
            print_response_with_parsed_value(&txt, cli.parse_value, cli.field.as_deref())?;
        }
        Commands::GetByIds { ids } => {
            let ids_vec: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
            let req = SecretsGetRequest { ids: ids_vec };
            let url = format!("{}/secrets-by-ids", base);
            let resp = client
                .get(&url)
                .headers(headers)
                .json(&req)
                .send()
                .await
                .context("request failed")?;
            let txt = resp.text().await?;
            print_response_with_parsed_value(&txt, cli.parse_value, cli.field.as_deref())?;
        }
        Commands::Create { key, value, note, project_ids } => {
            let pids = project_ids.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
            let req = SecretCreateRequest { key, value, note, organization_id: Some(org_id.clone()), project_ids: pids };
            let url = format!("{}/secret", base);
            let resp = client
                .post(&url)
                .headers(headers)
                .json(&req)
                .send()
                .await
                .context("request failed")?;
            let txt = resp.text().await?;
            print_response_with_parsed_value(&txt, cli.parse_value, cli.field.as_deref())?;
        }
        Commands::Update { id, key, value, note, project_ids } => {
            let pids = project_ids.map(|s| s.split(',').map(|s| s.trim().to_string()).collect());
            let req = SecretPutRequest { id, key, value, note, organization_id: Some(org_id.clone()), project_ids: pids };
            let url = format!("{}/secret", base);
            let resp = client
                .put(&url)
                .headers(headers)
                .json(&req)
                .send()
                .await
                .context("request failed")?;
            let txt = resp.text().await?;
            print_response_with_parsed_value(&txt, cli.parse_value, cli.field.as_deref())?;
        }
        Commands::Delete { ids } => {
            let ids_vec: Vec<String> = ids.split(',').map(|s| s.trim().to_string()).collect();
            let req = SecretsDeleteRequest { ids: ids_vec };
            let url = format!("{}/secret", base);
            let resp = client
                .delete(&url)
                .headers(headers)
                .json(&req)
                .send()
                .await
                .context("request failed")?;
            let txt = resp.text().await?;
            println!("{}", txt);
        }
        Commands::Render { file } => {
            let input = read_input(file)?;
            let output = render_template(
                input,
                &client,
                &headers,
                &base,
                &org_id,
                cli.verbose,
            )
            .await?;
            println!("{}", output);
        }
    }

    Ok(())
}
