use std::collections::HashMap;

use compact_str::CompactString;
use serde::{Deserialize, Serialize};

/// A sidecar config file holding only tool output limits. Loaded from
/// `local-limits-config.toml` next to the main config and merged on top
/// of it field-by-field. Intended as a convenience for local-LLM users
/// who run with tight context windows and want to swap in a tighter set
/// of limits without editing their main config.
///
/// `deny_unknown_fields` is intentional: putting anything other than the
/// five limit fields in this file is a parse error, so the file's purpose
/// stays machine-checked rather than convention-only.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalLimitsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_read_lines: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_bash_output_lines: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_grep_results: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_find_results: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_list_dir_entries: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickModelConfig {
    pub provider: CompactString,
    pub model: CompactString,
    #[serde(default)]
    pub input_token_cost: f64,
    #[serde(default)]
    pub output_token_cost: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiStyle {
    Responses,
    Completions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomProviderConfig {
    pub provider_type: CompactString,
    pub base_url: String,
    pub api_key_env: Option<CompactString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub danger_accept_invalid_certs: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_style: Option<ApiStyle>,
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub headers: HashMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<CompactString>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EditSystem {
    #[default]
    Similarity,
    Hashedit,
}

impl std::fmt::Display for EditSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EditSystem::Similarity => write!(f, "similarity"),
            EditSystem::Hashedit => write!(f, "hashedit"),
        }
    }
}

impl std::str::FromStr for EditSystem {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "similarity" => Ok(EditSystem::Similarity),
            "hashedit" => Ok(EditSystem::Hashedit),
            _ => Err(format!(
                "unknown edit system '{}' (valid: similarity, hashedit)",
                s
            )),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorsConfig {
    pub chat_background: Option<CompactString>,
    pub input_background: Option<CompactString>,
    pub status_background: Option<CompactString>,
}
