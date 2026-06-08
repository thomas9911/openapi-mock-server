use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;
use crate::fake_gen::generate_from_schema;

pub fn fallback_handler(
    state: Arc<RwLock<AppState>>,
) -> impl Fn(Request<Body>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Response> + Send>>
       + Clone
       + Send
       + 'static {
    move |req: Request<Body>| {
        let state = state.clone();
        Box::pin(async move {
            let method = req.method().clone();
            let path = req.uri().path().to_string();

            let st = state.read().await;

            if st.routes.is_empty() {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(serde_json::json!({
                        "error": "Not initialized. POST /_initialize with an OpenAPI spec first."
                    })),
                )
                    .into_response();
            }

            let matched = find_route(&st.routes, &method, &path);

            match matched {
                Some((route, path_params)) => {
                    if let Some(example) = &route.response_example {
                        return (StatusCode::OK, Json(example.clone())).into_response();
                    }

                    if let Some(schema) = &route.response_schema {
                        let generated = generate_fake_response(schema, &st.raw_spec, &path_params);
                        return (StatusCode::OK, Json(generated)).into_response();
                    }

                    (StatusCode::OK, Json(serde_json::json!({}))).into_response()
                }
                None => (
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({
                        "error": format!("No route found for {} {}", method, path)
                    })),
                )
                    .into_response(),
            }
        })
    }
}

fn find_route<'a>(
    routes: &'a [crate::state::RouteSpec],
    method: &Method,
    path: &str,
) -> Option<(&'a crate::state::RouteSpec, HashMap<String, String>)> {
    let method_str = method.as_str();

    // Exact match first (no path params)
    if let Some(r) = routes.iter().find(|r| r.method == method_str && r.path == path) {
        return Some((r, HashMap::new()));
    }

    // Path param match
    for r in routes.iter().filter(|r| r.method == method_str) {
        if let Some(params) = extract_path_params(&r.path, path) {
            return Some((r, params));
        }
    }

    None
}

fn extract_path_params(template: &str, actual: &str) -> Option<HashMap<String, String>> {
    let t_parts: Vec<&str> = template.split('/').collect();
    let a_parts: Vec<&str> = actual.split('/').collect();

    if t_parts.len() != a_parts.len() {
        return None;
    }

    let mut params = HashMap::new();
    for (t, a) in t_parts.iter().zip(a_parts.iter()) {
        if t.starts_with('{') && t.ends_with('}') {
            let param_name = &t[1..t.len() - 1];
            params.insert(param_name.to_string(), a.to_string());
        } else if *t != *a {
            return None;
        }
    }

    Some(params)
}

fn generate_fake_response(
    schema: &serde_json::Value,
    raw_spec: &Option<serde_json::Value>,
    path_params: &HashMap<String, String>,
) -> serde_json::Value {
    // Resolve top-level $ref
    if let Some(ref_str) = schema.get("$ref").and_then(serde_json::Value::as_str) {
        if let Some(spec) = raw_spec {
            if let Some(resolved) = resolve_ref_in_spec(ref_str, spec) {
                return inject_path_params(generate_from_schema(&resolved, None), path_params);
            }
        }
    }

    let resolved = resolve_schema_refs(schema, raw_spec);
    inject_path_params(generate_from_schema(&resolved, None), path_params)
}

/// Overwrite fields in the generated object whose names match path param names.
/// E.g. path param `petId=123` → sets field `petId` or `id` to 123.
fn inject_path_params(
    mut value: serde_json::Value,
    path_params: &HashMap<String, String>,
) -> serde_json::Value {
    if path_params.is_empty() {
        return value;
    }

    if let Some(obj) = value.as_object_mut() {
        for (param_name, param_val) in path_params {
            let coerced = coerce_param(param_val);

            // Exact field name match
            if obj.contains_key(param_name.as_str()) {
                obj.insert(param_name.clone(), coerced.clone());
            }

            // param is "fooId" or "foo_id" → also set field "id"
            let lower = param_name.to_lowercase();
            if (lower.ends_with("id") && lower.len() > 2) && obj.contains_key("id") {
                obj.insert("id".to_string(), coerced.clone());
            }
        }
    }

    value
}

fn coerce_param(val: &str) -> serde_json::Value {
    if let Ok(n) = val.parse::<i64>() {
        serde_json::Value::Number(n.into())
    } else {
        serde_json::Value::String(val.to_string())
    }
}

fn resolve_ref_in_spec(ref_str: &str, spec: &serde_json::Value) -> Option<serde_json::Value> {
    if !ref_str.starts_with('#') {
        return None;
    }

    let parts: Vec<&str> = ref_str
        .trim_start_matches('#')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    let mut current = spec;
    for part in &parts {
        current = current.get(*part)?;
    }

    Some(resolve_schema_refs(current, &Some(spec.clone())))
}

fn resolve_schema_refs(schema: &serde_json::Value, spec: &Option<serde_json::Value>) -> serde_json::Value {
    if let Some(ref_str) = schema.get("$ref").and_then(serde_json::Value::as_str) {
        if let Some(s) = spec {
            if let Some(resolved) = resolve_ref_in_spec(ref_str, s) {
                return resolved;
            }
        }
        return schema.clone();
    }

    match schema {
        serde_json::Value::Object(map) => {
            let mut new_map = serde_json::Map::new();
            for (k, v) in map {
                if k == "properties" {
                    if let serde_json::Value::Object(props) = v {
                        let mut new_props = serde_json::Map::new();
                        for (pk, pv) in props {
                            new_props.insert(pk.clone(), resolve_schema_refs(pv, spec));
                        }
                        new_map.insert(k.clone(), serde_json::Value::Object(new_props));
                    } else {
                        new_map.insert(k.clone(), v.clone());
                    }
                } else if k == "items" || k == "additionalProperties" {
                    new_map.insert(k.clone(), resolve_schema_refs(v, spec));
                } else {
                    new_map.insert(k.clone(), v.clone());
                }
            }
            serde_json::Value::Object(new_map)
        }
        _ => schema.clone(),
    }
}
