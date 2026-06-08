use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::{AppState, RouteSpec};

pub async fn handle_initialize(
    State(state): State<Arc<RwLock<AppState>>>,
    body: String,
) -> impl IntoResponse {
    let spec: Value = match serde_json::from_str(&body)
        .or_else(|_| yaml_serde::from_str::<Value>(&body).map_err(|e| e.to_string()))
    {
        Ok(v) => v,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "error": format!("Failed to parse spec: {}", e) })),
            );
        }
    };

    let routes = extract_routes(&spec);
    let route_count = routes.len();

    let mut st = state.write().await;
    st.routes = routes;
    st.raw_spec = Some(spec);

    tracing::info!("Initialized with {} routes", route_count);

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "initialized",
            "routes": route_count
        })),
    )
}

fn extract_routes(spec: &Value) -> Vec<RouteSpec> {
    let mut routes = Vec::new();

    let paths = match spec.get("paths").and_then(Value::as_object) {
        Some(p) => p,
        None => return routes,
    };

    for (path, path_item) in paths {
        let methods = ["get", "post", "put", "patch", "delete", "head", "options"];

        for method in methods {
            let operation = match path_item.get(method) {
                Some(op) => op,
                None => continue,
            };

            let (response_schema, response_example) = extract_success_response(operation, spec);

            routes.push(RouteSpec {
                method: method.to_uppercase(),
                path: path.clone(),
                response_schema,
                response_example,
            });
        }
    }

    routes
}

fn extract_success_response(operation: &Value, spec: &Value) -> (Option<Value>, Option<Value>) {
    let responses = match operation.get("responses").and_then(Value::as_object) {
        Some(r) => r,
        None => return (None, None),
    };

    // Prefer 200, then 201, then first 2xx
    let success_codes = ["200", "201", "202", "204", "203"];
    let mut chosen = None;

    for code in success_codes {
        if let Some(resp) = responses.get(code) {
            chosen = Some(resp);
            break;
        }
    }

    if chosen.is_none() {
        chosen = responses
            .iter()
            .find(|(k, _)| k.starts_with('2'))
            .map(|(_, v)| v);
    }

    let resp = match chosen {
        Some(r) => r,
        None => return (None, None),
    };

    // Resolve $ref if needed
    let resp = resolve_ref(resp, spec).unwrap_or(resp);

    // v2: response schema is directly on the response object
    if let Some(schema) = resp.get("schema") {
        let resolved_schema = resolve_ref(schema, spec)
            .map(|s| s.clone())
            .or_else(|| Some(schema.clone()));
        return (resolved_schema, None);
    }

    // v3: schema is under content.application/json
    let content = resp
        .get("content")
        .and_then(|c| c.get("application/json"))
        .or_else(|| {
            resp.get("content")
                .and_then(|c| c.as_object())
                .and_then(|m| m.values().next())
        });

    let media = match content {
        Some(m) => m,
        None => return (None, None),
    };

    // Check for example
    let example = media.get("example").cloned().or_else(|| {
        media
            .get("examples")
            .and_then(Value::as_object)
            .and_then(|ex| ex.values().next())
            .and_then(|ex| ex.get("value"))
            .cloned()
    });

    if example.is_some() {
        return (None, example);
    }

    // Fall back to schema
    let schema = media.get("schema").cloned();
    let resolved_schema = schema
        .as_ref()
        .and_then(|s| resolve_ref(s, spec))
        .map(|s| s.clone())
        .or(schema);

    (resolved_schema, None)
}

fn resolve_ref<'a>(value: &'a Value, spec: &'a Value) -> Option<&'a Value> {
    let ref_str = value.get("$ref").and_then(Value::as_str)?;

    // Only handle local refs like #/components/schemas/Foo
    if !ref_str.starts_with('#') {
        return None;
    }

    let parts: Vec<&str> = ref_str
        .trim_start_matches('#')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();
    let mut current = spec;
    for part in parts {
        current = current.get(part)?;
    }
    Some(current)
}
