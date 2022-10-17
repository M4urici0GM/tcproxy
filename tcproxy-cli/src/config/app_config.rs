use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct AppContext {
    name: String,
    target_ip: String,
}

#[derive(Debug, Serialize)]
pub struct AppConfig {
    default_context: String,
    contexts: Vec<AppContext>,
}
