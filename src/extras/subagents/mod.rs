use std::sync::Mutex;

use crate::provider::AnyClient;

pub(crate) mod builder;
pub(crate) mod prompt;
pub(crate) mod task_tool;

pub(crate) struct SubagentConfig {
    pub client: AnyClient,
    pub model_name: String,
    pub max_turns: usize,
    #[cfg(feature = "archmd")]
    pub architecture: Option<String>,
}

static CONFIG: Mutex<Option<SubagentConfig>> = Mutex::new(None);

pub(crate) fn with_config<F, R>(f: F) -> R
where
    F: FnOnce(&SubagentConfig) -> R,
{
    let guard = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    let cfg = guard
        .as_ref()
        .expect("subagents: SubagentConfig not initialized (call init() in main.rs)");
    f(cfg)
}

pub fn init(
    client: AnyClient,
    model_name: String,
    max_turns: usize,
    #[cfg(feature = "archmd")] architecture: Option<String>,
) {
    let mut guard = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(SubagentConfig {
        client,
        model_name,
        max_turns,
        #[cfg(feature = "archmd")]
        architecture,
    });
}

pub fn set_client_and_model(client: AnyClient, model_name: String) {
    let mut guard = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(cfg) = guard.as_mut() {
        cfg.client = client;
        cfg.model_name = model_name;
    }
}

pub fn set_model_name(model_name: String) {
    let mut guard = CONFIG.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(cfg) = guard.as_mut() {
        cfg.model_name = model_name;
    }
}
