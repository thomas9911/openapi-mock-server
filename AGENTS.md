# AGENTS.md

Guidelines for AI agents working on this codebase.

## Project

Rust service (tokio + axum) that mocks an OpenAPI v3 API. Takes a spec via `POST /_initialize`, then serves fake responses for every path defined in the spec.

## Architecture

```
src/
  main.rs        — server setup, route registration
  state.rs       — shared AppState (routes + raw spec), RouteSpec struct
  initialize.rs  — POST /_initialize handler: parses spec, extracts routes
  mock_router.rs — fallback handler: matches requests, injects path params, generates responses
  fake_gen.rs    — fake data generation from JSON Schema
  docs.rs        — GET /_spec and GET /_docs (Swagger UI)
```

## Key flows

**Initialization** (`initialize.rs`):
1. Parse body as JSON, fall back to YAML
2. Walk `paths` → for each method, find the first 2xx response
3. Prefer `example` over `schema` in the response content
4. Store `Vec<RouteSpec>` and the raw spec in `AppState`

**Request matching** (`mock_router.rs`):
1. Exact path match first
2. Then template match — `{param}` segments match any value
3. Extract path params as `HashMap<String, String>`
4. Generate response, then overwrite fields matching param names

**Fake generation** (`fake_gen.rs`):
- `generate_from_schema(schema, field_name)` is the entry point
- Field name heuristics run before type-based generation
- `$ref` resolution happens in `mock_router.rs` before calling `fake_gen`
- `additionalProperties` generates a fixed set of keys with typed values

## Conventions

- No auth on mock endpoints — all security schemes in the spec are ignored
- `/_initialize` is protected by `API_KEY` env var if set; key is passed as plain `Authorization` header
- `$ref` only resolves local refs (`#/components/...`); external refs return null
- Path params are coerced: numeric strings → JSON number, else string
- `name` alone is too generic for a person name — falls through to lorem word
- `additionalProperties` without `properties` generates keys: `available`, `pending`, `sold`

## Common tasks

**Add a new field name heuristic** — edit `generate_string_by_field_name` in `fake_gen.rs`. Pattern: check `name.contains("x")` and return a `faker::...` value.

**Add a new format handler** — edit the `fmt` match in `generate_string` in `fake_gen.rs`.

**Change port** — pass `--port <n>` CLI flag or set `PORT=<n>` env var. CLI takes precedence. Default is `3000`.

**Enable API key protection** — set `API_KEY=<key>` env var on startup. The key is stored in `AppState.api_key` and checked in `handle_initialize` against the plain `Authorization` header. If unset, `/_initialize` is open.

**Add a new internal endpoint** — register a `get`/`post` route in `main.rs` before the fallback, add a handler in the appropriate module or a new file, wire up `with_state` if it needs `AppState`.

**OpenAPI v2 (Swagger 2.0)** — supported. v2 response schemas sit directly on the response object (`responses.200.schema`); v3 wraps them under `content.application/json.schema`. Both are handled in `extract_success_response`. `$ref` resolution works for both `#/definitions/` and `#/components/schemas/` since it just traverses JSON keys.

**Swap the spec at runtime** — re-POST to `/_initialize`. The existing routes are replaced atomically via `RwLock` write lock.

**Support a new HTTP method** — add it to the `methods` array in `extract_routes` in `initialize.rs`.

## Build & run

```bash
cargo build
cargo run

# initialize
curl -X POST http://localhost:3000/_initialize -H 'Content-Type: application/json' -d @openapi.json

# or
./initialize.sh
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `axum` | HTTP server + routing |
| `tokio` | Async runtime |
| `serde_json` | JSON parsing and generation |
| `yaml_serde` | YAML spec parsing |
| `fake` | Fake data generation |
| `rand` | Random number generation |
| `uuid` | UUID generation for id fields |
| `chrono` | date-time format generation |
