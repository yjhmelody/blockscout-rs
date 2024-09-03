use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TracingFormat {
    Default,
    Json,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct TracingSettings {
    /// If disabled, tracing is not initialized for neither
    /// stdout, nor jaeger (enabled by default).
    pub enabled: bool,
    pub format: TracingFormat,
}

impl Default for TracingSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            format: TracingFormat::Default,
        }
    }
}
