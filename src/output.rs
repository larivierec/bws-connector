use serde_json::Value as JsonValue;

use crate::render::extract_path;

/// Print a response, optionally parsing value field as JSON and extracting a specific field
pub fn print_response_with_parsed_value(
    txt: &str,
    parse_value: bool,
    field: Option<&str>,
) -> anyhow::Result<()> {
    // If no JSON parsing requested and no field extraction, just print raw
    if !parse_value && field.is_none() {
        println!("{}", txt);
        return Ok(());
    }

    // Try to parse top-level response JSON
    let mut v: JsonValue = match serde_json::from_str(txt) {
        Ok(j) => j,
        Err(_) => {
            if field.is_some() {
                return Err(anyhow::anyhow!("response is not valid JSON, cannot extract field"));
            }
            // not JSON and no field requested: print raw
            println!("{}", txt);
            return Ok(());
        }
    };

    // If there's a `value` string, try to parse it as JSON and replace it with parsed JSON
    if let Some(val) = v.get_mut("value") {
        if val.is_string() {
            if let Some(s) = val.as_str() {
                if let Ok(parsed) = serde_json::from_str::<JsonValue>(s) {
                    *val = parsed;
                }
            }
        }
    }

    // If `data` array exists, attempt same transformation on each element
    if let Some(data) = v.get_mut("data") {
        if let Some(array) = data.as_array_mut() {
            for item in array.iter_mut() {
                if let Some(val) = item.get_mut("value") {
                    if val.is_string() {
                        if let Some(s) = val.as_str() {
                            if let Ok(parsed) = serde_json::from_str::<JsonValue>(s) {
                                *val = parsed;
                            }
                        }
                    }
                }
            }
        }
    }

    // If a specific field was requested, attempt to extract it and print only that
    if let Some(path) = field {
        // Try to extract from top-level value first
        let mut extracted: Option<JsonValue> = None;
        if let Some(val) = v.get("value") {
            extracted = extract_path(val, path);
        }

        // If not found top-level, try data array first-match
        if extracted.is_none() {
            if let Some(data) = v.get("data") {
                if let Some(array) = data.as_array() {
                    for item in array.iter() {
                        if let Some(val) = item.get("value") {
                            if let Some(found) = extract_path(val, path) {
                                extracted = Some(found);
                                break;
                            }
                        }
                    }
                }
            }
        }

        if let Some(found) = extracted {
            // If the found value is a string, print raw (no quotes) for templates
            if found.is_string() {
                println!("{}", found.as_str().unwrap());
            } else {
                println!("{}", serde_json::to_string_pretty(&found)?);
            }
            return Ok(());
        } else {
            return Err(anyhow::anyhow!("field not found: {}", path));
        }
    }

    // If parsing was requested but no specific field, print full parsed JSON
    if parse_value {
        println!("{}", serde_json::to_string_pretty(&v)?);
    } else {
        println!("{}", txt);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_inner_value_string_to_json() {
        // outer JSON has 'value' as a JSON-encoded string
        let txt = r#"{"value":"{\"inner\": {\"k\": \"v\"}}"}"#;
        let mut v: JsonValue = serde_json::from_str(txt).expect("parse outer");
        if let Some(val) = v.get_mut("value") {
            if val.is_string() {
                if let Some(s) = val.as_str() {
                    let parsed: JsonValue = serde_json::from_str(s).expect("parse inner");
                    *val = parsed;
                }
            }
        }

        // now extract inner.k
        let got = extract_path(&v["value"], "inner.k").expect("found path");
        assert_eq!(got, serde_json::json!("v"));
    }
}
