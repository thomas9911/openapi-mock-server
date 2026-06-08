# swagger-mock-server

A Rust service that mocks an API based on an OpenAPI v2 or v3 spec. It generates realistic fake data from schemas using field name heuristics (emails, names, phone numbers, UUIDs, etc.).

## Usage

### Start the server

```bash
cargo run
# CLI flag (takes precedence):
cargo run -- --port 8080
# or via env var:
PORT=8080 cargo run
```

Listens on `http://0.0.0.0:3000` by default.

### Initialize with a spec

```bash
curl -X POST http://localhost:3000/_initialize \
  -H 'Content-Type: application/json' \
  -d @openapi.json
```

Or use the included script:

```bash
./initialize.sh
```

Accepts JSON or YAML, OpenAPI v2 or v3. Re-POST at any time to swap the spec.

### Call mock endpoints

All paths defined in the spec are now available and return generated data:

```bash
curl http://localhost:3000/pet/123
# → { "id": 123, "name": "laborum", "status": "available", ... }

curl http://localhost:3000/users
# → [{ "id": "...", "email": "jane@example.com", "firstName": "Jane", ... }]
```

**Path parameters** are injected into the response. `GET /pet/123` returns `id: 123`.

**Response selection:** the first 2xx response defined in the spec is used. If the response defines an example, that is returned as-is. Otherwise fake data is generated from the schema.

### API docs

```
GET /_docs   → Swagger UI
GET /_spec   → Raw OpenAPI JSON (servers rewritten to point to this mock)
```

## Fake data generation

| Field name pattern | Generated value |
|--------------------|----------------|
| `email`, `*mail*` | `jane@example.com` |
| `firstName`, `first_name` | `Jane` |
| `lastName`, `last_name` | `Doe` |
| `fullName`, `displayName` | `Jane Doe` |
| `username`, `login` | `jane_doe` |
| `phone`, `mobile` | `+1-555-123-4567` |
| `id`, `*Id`, `*_id` | UUID |
| `city` | `Amsterdam` |
| `country` | `Netherlands` |
| `zipCode`, `postalCode` | `1234 AB` |
| `company`, `organization` | `Acme Corp` |
| `description`, `bio` | Lorem sentence |
| `content`, `body`, `message` | Lorem paragraph |

Format hints in the schema (`format: email`, `date-time`, `uuid`, `uri`) take precedence over field name heuristics.

## Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/_initialize` | Load an OpenAPI v3 spec (JSON or YAML) |
| `GET` | `/_docs` | Swagger UI |
| `GET` | `/_spec` | Current spec JSON |
| `*` | everything else | Mocked responses |
