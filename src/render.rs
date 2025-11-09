use anyhow::Context;
use regex::Regex;
use reqwest::header::HeaderMap;
use serde_json::Value as JsonValue;
use std::path::PathBuf;

use crate::models::{ListResponse, SecretGetRequest};

/// Render placeholders in input text, replacing bws://key[/path] with secret values
pub async fn render_template(
    input: String,
    client: &reqwest::Client,
    headers: &HeaderMap,
    base_url: &str,
    org_id: &str,
    verbose: bool,
) -> anyhow::Result<String> {
    let re = Regex::new(r"bws://([A-Za-z0-9_\-]+)(?:/([A-Za-z0-9_./-]+))?").unwrap();
    let mut out = input.clone();

    for cap in re.captures_iter(&input) {
        let placeholder = cap.get(0).unwrap().as_str();
        let key = cap.get(1).map(|m| m.as_str()).unwrap();
        let path = cap.get(2).map(|m| m.as_str());
        let ph_start = cap.get(0).unwrap().start();
        let ph_end = cap.get(0).unwrap().end();

        // Fetch list to find secret id by key
        let url = format!("{}/secrets", base_url);
        let resp = client
            .get(&url)
            .headers(headers.clone())
            .json(&serde_json::json!({"OrganizationID": org_id}))
            .send()
            .await
            .context("request failed")?;
        
        let txt = resp.text().await?;
        if verbose {
            eprintln!("raw list response: {}", txt);
        }

        let list: ListResponse = serde_json::from_str(&txt).context("failed to parse list response")?;
        if verbose {
            let keys: Vec<String> = list.data.iter().map(|i| i.key.clone()).collect();
            eprintln!("found keys: {:?}", keys);
        }

        if let Some(found) = list.data.into_iter().find(|i| i.key == key) {
            let get_req = SecretGetRequest { id: found.id };
            let url = format!("{}/secret", base_url);
            let resp = client
                .get(&url)
                .headers(headers.clone())
                .json(&get_req)
                .send()
                .await
                .context("request failed")?;
            
            let secret_txt = resp.text().await?;
            
            if let Ok(json) = serde_json::from_str::<JsonValue>(&secret_txt) {
                // Prefer the `value` field, otherwise use the whole JSON
                let val_ref = json.get("value").unwrap_or(&json);

                // If `value` is a string that contains JSON, try to parse it
                let parsed_val: JsonValue = if val_ref.is_string() {
                    if let Some(s) = val_ref.as_str() {
                        match serde_json::from_str::<JsonValue>(s) {
                            Ok(inner) => inner,
                            Err(_) => val_ref.clone(),
                        }
                    } else {
                        val_ref.clone()
                    }
                } else {
                    val_ref.clone()
                };

                // If no explicit path was provided, default to extracting value.<key>
                let target_path = path.map(|p| p.to_string()).unwrap_or_else(|| key.to_string());

                if verbose {
                    eprintln!(
                        "extracting path '{}' from parsed value: {}",
                        target_path,
                        serde_json::to_string(&parsed_val).unwrap_or_else(|_| "<unprintable>".to_string())
                    );
                }

                let rep = extract_path(&parsed_val, &target_path)
                    .map(|v| {
                        if v.is_string() {
                            v.as_str().unwrap().to_string()
                        } else {
                            serde_json::to_string(&v).unwrap()
                        }
                    })
                    .or_else(|| {
                        // If no explicit path was provided and extraction failed, fall back to whole value
                        if path.is_none() {
                            Some(if parsed_val.is_string() {
                                parsed_val.as_str().unwrap().to_string()
                            } else {
                                serde_json::to_string(&parsed_val).unwrap()
                            })
                        } else {
                            None
                        }
                    });

                if let Some(replacement) = rep {
                    // If the placeholder sits alone on an indented line (common with YAML | or |- blocks),
                    // indent each line of a multiline replacement to match the placeholder indentation.
                    let adjusted = if replacement.contains('\n') {
                        // find start of the line containing the placeholder in the original input
                        let line_start = input[..ph_start].rfind('\n').map(|i| i + 1).unwrap_or(0);
                        let line_end = input[ph_end..].find('\n').map(|i| ph_end + i).unwrap_or(input.len());
                        let line = &input[line_start..line_end];
                        // compute indentation (spaces/tabs) at start of the line
                        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
                        // check whether the placeholder is the only non-whitespace content on that line
                        if line.trim() == placeholder {
                            // indent every line of the replacement with the same indentation
                            let indented = replacement
                                .lines()
                                .map(|l| format!("{}{}", indent, l))
                                .collect::<Vec<_>>()
                                .join("\n");
                            indented
                        } else {
                            // placeholder is inline with other content; leave replacement unchanged
                            replacement
                        }
                    } else {
                        replacement
                    };

                    out = out.replacen(placeholder, &adjusted, 1);
                }
            }
        }
    }

    Ok(out)
}

/// Read input from file or stdin
pub fn read_input(file: Option<PathBuf>) -> anyhow::Result<String> {
    if let Some(path) = file {
        std::fs::read_to_string(path).context("reading input file")
    } else {
        let mut s = String::new();
        use std::io::Read;
        std::io::stdin().read_to_string(&mut s).context("reading stdin")?;
        Ok(s)
    }
}

/// Extract a field from a JSON value by path (dot or slash separated)
pub fn extract_path(val: &JsonValue, path: &str) -> Option<JsonValue> {
    let sep = if path.contains('/') { '/' } else { '.' };
    let mut cur = val;
    for part in path.split(sep) {
        if let Some(next) = cur.get(part) {
            cur = next;
        } else {
            return None;
        }
    }
    Some(cur.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_path_dot() {
        let json: JsonValue = serde_json::json!({"value": {"a": {"b": 42}}});
        let v = extract_path(&json["value"], "a.b");
        assert!(v.is_some());
        assert_eq!(v.unwrap(), serde_json::json!(42));
    }

    #[test]
    fn test_extract_path_slash() {
        let json: JsonValue = serde_json::json!({"value": {"a": {"b": "hello"}}});
        let v = extract_path(&json["value"], "a/b");
        assert!(v.is_some());
        assert_eq!(v.unwrap(), serde_json::json!("hello"));
    }

    #[test]
    fn test_bws_regex_captures() {
        let re = Regex::new(r"bws://([A-Za-z0-9_\-]+)(?:/([A-Za-z0-9_./-]+))?").unwrap();
        let caps = re.captures("bws://minio_tf_volsync/secret_key").unwrap();
        assert_eq!(&caps[1], "minio_tf_volsync");
        assert_eq!(&caps[2], "secret_key");

        let caps2 = re.captures("bws://simplekey").unwrap();
        assert_eq!(&caps2[1], "simplekey");
        assert!(caps2.get(2).is_none());
    }
}
