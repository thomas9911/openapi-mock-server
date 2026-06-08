use axum::{
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::state::AppState;

pub async fn handle_spec(State(state): State<Arc<RwLock<AppState>>>) -> impl IntoResponse {
    let st = state.read().await;
    match &st.raw_spec {
        Some(spec) => {
            let mut spec = spec.clone();
            // Override servers so Swagger UI calls our mock, not the original API
            if let Some(obj) = spec.as_object_mut() {
                obj.insert(
                    "servers".to_string(),
                    serde_json::json!([{ "url": "/", "description": "Mock server" }]),
                );
            }
            (StatusCode::OK, Json(spec)).into_response()
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Not initialized yet." })),
        )
            .into_response(),
    }
}

pub async fn handle_docs() -> Response {
    let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>API Docs</title>
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist/swagger-ui.css">
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist/swagger-ui-bundle.js"></script>
  <script>
    SwaggerUIBundle({
      url: '/_spec',
      dom_id: '#swagger-ui',
      presets: [SwaggerUIBundle.presets.apis, SwaggerUIBundle.SwaggerUIStandalonePreset],
      layout: 'BaseLayout',
      tryItOutEnabled: false,
    });
  </script>
</body>
</html>"#;

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        html,
    )
        .into_response()
}
