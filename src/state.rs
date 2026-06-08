use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RouteSpec {
    pub method: String,
    pub path: String,
    pub response_schema: Option<Value>,
    pub response_example: Option<Value>,
}

#[derive(Debug, Default)]
pub struct AppState {
    pub routes: Vec<RouteSpec>,
    pub raw_spec: Option<Value>,
}
