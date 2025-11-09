# Code Structure

This document describes the organization of the bitwarden-connect CLI codebase.

## Module Overview

The codebase is organized into focused modules, each with a single responsibility:

### `src/main.rs`
- Entry point for the application
- Parses CLI arguments and dispatches to command handlers
- Coordinates between modules (client setup, command execution, output)
- Handles the main async runtime

### `src/cli.rs`
- CLI structure and command definitions using `clap`
- Defines all available commands: `get`, `get-by-key`, `list`, `get-by-ids`, `create`, `update`, `delete`, `render`
- Command arguments and global flags (base-url, access-token, TLS options, verbose, etc.)

### `src/models.rs`
- Request and response data structures
- Serialization/deserialization with `serde`
- Types include:
  - `SecretGetRequest`, `SecretsGetRequest`, `SecretsDeleteRequest`
  - `SecretCreateRequest`, `SecretPutRequest`
  - `ListItem`, `ListResponse`

### `src/client.rs`
- HTTP client builder with TLS customization (`build_client`)
- Header construction for Warden authentication (`build_headers`)
- Handles `--insecure` and `--ca-cert` options

### `src/render.rs`
- Template rendering logic for `bws://` placeholders
- Main function: `render_template` - replaces placeholders with secret values
- Helper functions:
  - `read_input` - reads from file or stdin
  - `extract_path` - extracts nested JSON fields using dot or slash notation
- Unit tests for regex matching and path extraction

### `src/output.rs`
- Response formatting and printing
- `print_response_with_parsed_value` - handles output with optional JSON parsing and field extraction
- Supports `--parse-value` and `--field` flags
- Unit tests for JSON value parsing

## Data Flow

1. **CLI Parsing** (`main.rs` → `cli.rs`)
   - User input is parsed into `Cli` struct with command and flags

2. **Client Setup** (`main.rs` → `client.rs`)
   - HTTP client is built with TLS options
   - Headers are constructed with Warden authentication

3. **Command Execution** (`main.rs` → various modules)
   - Commands dispatch to appropriate handlers
   - Most commands make HTTP requests directly in `main.rs`
   - `render` command delegates to `render.rs`

4. **Output** (`main.rs` → `output.rs`)
   - Responses are formatted and printed
   - Supports pretty-printing, field extraction, and raw output

## Testing

- Tests are co-located with their modules using `#[cfg(test)]`
- Run all tests: `cargo test`
- Current test coverage:
  - Path extraction (dot and slash notation)
  - JSON value parsing
  - Regex placeholder matching

## Adding New Commands

1. Add command variant to `Commands` enum in `src/cli.rs`
2. Add command handler in the match statement in `src/main.rs`
3. Create any needed request/response types in `src/models.rs`
4. Use `print_response_with_parsed_value` for consistent output formatting

## Dependencies

- `clap` - CLI parsing
- `reqwest` - HTTP client (with `rustls` for TLS)
- `serde`, `serde_json` - JSON serialization
- `tokio` - Async runtime
- `anyhow` - Error handling
- `regex` - Placeholder matching in templates
