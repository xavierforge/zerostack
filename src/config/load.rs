use std::collections::HashMap;
use std::path::PathBuf;

use compact_str::CompactString;

use crate::config::{Config, EditSystem, LocalLimitsConfig, QuickModelConfig, ShowToolDetails};
#[cfg(feature = "mcp")]
use crate::extras::mcp::config::McpServerConfig;
use crate::session::storage;

fn resolve_config_path() -> PathBuf {
    if let Some(dir) = std::env::var_os("ZS_CONFIG_DIR") {
        let dir = PathBuf::from(dir);
        let toml = dir.join("config.toml");
        let json = dir.join("config.json");
        if toml.exists() {
            return toml;
        }
        if json.exists() {
            return json;
        }
        return toml;
    }

    if let Some(config_dir) = dirs::config_dir() {
        let dir = config_dir.join("zerostack");
        let toml = dir.join("config.toml");
        let json = dir.join("config.json");
        if toml.exists() {
            return toml;
        }
        if json.exists() {
            return json;
        }
    }

    let dir = storage::data_dir();
    let toml = dir.join("config.toml");
    let json = dir.join("config.json");
    if toml.exists() {
        toml
    } else if json.exists() {
        json
    } else {
        toml
    }
}

pub fn config_file_path() -> PathBuf {
    resolve_config_path()
}

/// Path to the optional `local-limits-config.toml` sidecar, in the same
/// directory as the main config. Returns the candidate path whether or not
/// the file exists — callers check existence separately.
pub fn local_limits_config_path() -> PathBuf {
    resolve_config_path()
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(storage::data_dir)
        .join("local-limits-config.toml")
}

/// Load `local-limits-config.toml` if it exists. Returns `None` when the
/// file is absent (the common case for non-local-LLM users). Parse errors
/// — including the `deny_unknown_fields` rejection for non-limit keys —
/// are fatal and printed to stderr before exit, matching the main config's
/// behaviour.
pub fn load_local_limits() -> Option<LocalLimitsConfig> {
    let path = local_limits_config_path();
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!(
            "error: failed to read {} ({}). \
             Remove or fix the file to continue.",
            path.display(),
            e,
        );
        std::process::exit(1);
    });
    let local: LocalLimitsConfig = toml::from_str(&content).unwrap_or_else(|e| {
        eprintln!(
            "error: {} is not a valid local-limits config: {}\n\
             This file may only contain max_read_lines, max_bash_output_lines, \
             max_grep_results, max_find_results, and max_list_dir_entries.",
            path.display(),
            e,
        );
        std::process::exit(1);
    });
    Some(local)
}

/// Apply a loaded `LocalLimitsConfig` on top of the main `Config`,
/// field-by-field. A `Some(_)` in the local config wins over the main
/// config's value (whether the main value is `Some` or `None`). A `None`
/// in the local config leaves the main config's value untouched.
pub fn merge_local_limits(cfg: &mut Config, local: &LocalLimitsConfig) {
    if local.max_read_lines.is_some() {
        cfg.max_read_lines = local.max_read_lines;
    }
    if local.max_bash_output_lines.is_some() {
        cfg.max_bash_output_lines = local.max_bash_output_lines;
    }
    if local.max_grep_results.is_some() {
        cfg.max_grep_results = local.max_grep_results;
    }
    if local.max_find_results.is_some() {
        cfg.max_find_results = local.max_find_results;
    }
    if local.max_list_dir_entries.is_some() {
        cfg.max_list_dir_entries = local.max_list_dir_entries;
    }
}

fn default_quick_models() -> HashMap<String, QuickModelConfig> {
    let mut map = HashMap::new();
    map.insert(
        "deepseek-v4-flash".to_string(),
        QuickModelConfig {
            provider: CompactString::new("openrouter"),
            model: CompactString::new("deepseek/deepseek-v4-flash"),
            input_token_cost: 0.0983,
            output_token_cost: 0.1966,
        },
    );
    map.insert(
        "deepseek-v4-pro".to_string(),
        QuickModelConfig {
            provider: CompactString::new("openrouter"),
            model: CompactString::new("deepseek/deepseek-v4-pro"),
            input_token_cost: 0.435,
            output_token_cost: 0.87,
        },
    );
    map
}

pub fn quick_models_map(cfg: &Config) -> HashMap<String, QuickModelConfig> {
    let mut map = default_quick_models();
    if let Some(user_models) = &cfg.quick_models {
        for (k, v) in user_models {
            map.insert(k.clone(), v.clone());
        }
    }
    map
}

pub fn save_quick_model(
    name: &str,
    provider: &str,
    model: &str,
    input_token_cost: f64,
    output_token_cost: f64,
) -> std::io::Result<()> {
    let path = resolve_config_path();
    let mut cfg: Config = if path.exists() {
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str(&content).unwrap_or_default(),
            _ => serde_json::from_str(&content).unwrap_or_default(),
        }
    } else {
        Config::default()
    };

    let quick_models = cfg.quick_models.get_or_insert_with(HashMap::new);
    quick_models.insert(
        name.to_string(),
        QuickModelConfig {
            provider: CompactString::new(provider),
            model: CompactString::new(model),
            input_token_cost,
            output_token_cost,
        },
    );

    let parent = path.parent().ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid config path")
    })?;
    std::fs::create_dir_all(parent)?;
    match path.extension().and_then(|e| e.to_str()) {
        Some("toml") => {
            let content = toml::to_string(&cfg).map_err(std::io::Error::other)?;
            std::fs::write(&path, content)?;
        }
        _ => std::fs::write(&path, serde_json::to_string_pretty(&cfg)?)?,
    }
    Ok(())
}

fn rich_default_config() -> Config {
    let mut cfg = Config::default();
    cfg.quick_models = Some(default_quick_models());
    cfg.provider = Some(CompactString::new("openrouter"));
    cfg.model = Some(CompactString::new("deepseek/deepseek-v4-pro"));
    cfg.max_tokens = Some(16384);
    cfg.context_window = Some(128_000);
    cfg.compact_enabled = Some(true);
    cfg.max_text_file_size = Some(1_048_576);
    cfg.edit_system = Some(EditSystem::Similarity);
    cfg.default_permission_mode = Some("standard".to_string());
    cfg.default_prompt = Some(CompactString::new("code"));
    cfg.show_tool_details = Some(ShowToolDetails::Lines(1));
    cfg.subagent_model = Some(CompactString::new("deepseek-v4-flash"));
    cfg
}

pub fn load() -> Config {
    let path = resolve_config_path();
    #[allow(unused_mut)]
    let mut cfg: Config = if !path.exists() {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let default = rich_default_config();
        if path.extension().and_then(|e| e.to_str()) == Some("toml")
            && let Ok(content) = toml::to_string(&default)
        {
            std::fs::write(&path, content).ok();
        }
        default
    } else {
        let content = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            eprintln!(
                "error: failed to read config file ({}): {}\n\
                 Fix the file or remove it to use defaults.",
                path.display(),
                e,
            );
            std::process::exit(1);
        });
        match path.extension().and_then(|e| e.to_str()) {
            Some("toml") => toml::from_str(&content).unwrap_or_else(|e| {
                eprintln!(
                    "error: {} is not a valid config: {}\n\
                     Fix the file or remove it to use defaults.",
                    path.display(),
                    e,
                );
                std::process::exit(1);
            }),
            _ => serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!(
                    "error: {} is not a valid config: {}\n\
                     Fix the file or remove it to use defaults.",
                    path.display(),
                    e,
                );
                std::process::exit(1);
            }),
        }
    };

    // Sidecar limits config wins field-by-field over the main config.
    if let Some(local) = load_local_limits() {
        merge_local_limits(&mut cfg, &local);
    }

    #[cfg(feature = "mcp")]
    if cfg.mcp_servers.is_none() {
        let mut headers = HashMap::new();
        if let Ok(key) = std::env::var("EXA_API_KEY") {
            headers.insert("x-api-key".to_string(), key);
        }
        let mut defaults = HashMap::new();
        defaults.insert(
            "Exa Web Search".to_string(),
            McpServerConfig::Url {
                url: "https://mcp.exa.ai/mcp".to_string(),
                headers,
            },
        );
        cfg.mcp_servers = Some(defaults);
    }

    cfg
}
